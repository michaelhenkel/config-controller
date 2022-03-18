package db

import (
	"github.com/michaelhenkel/config-controller/pkg/graph"
	"k8s.io/klog/v2"
	contrail "ssd-git.juniper.net/contrail/cn2/contrail/pkg/apis/core/v1alpha1"
)

type action string

const (
	add action = "add"
	del action = "del"
)

type HandlerInterface interface {
	GetReferences(obj interface{}) []contrail.ResourceReference
}

func (d *DB) AddHandlerInterface(kind string, handlerInterface HandlerInterface) {
	if d.handlerInterfaceMap == nil {
		d.handlerInterfaceMap = map[string]HandlerInterface{}
	}
	d.handlerInterfaceMap[kind] = handlerInterface
}

type control struct {
	action     action
	kind       string
	namespace  string
	name       string
	references [][]string
}

type DB struct {
	graph               graph.ItemGraph
	ctrlChan            chan control
	stopChan            chan struct{}
	handlerInterfaceMap map[string]HandlerInterface
}

func NewClient() *DB {
	return &DB{
		graph:    graph.ItemGraph{},
		ctrlChan: make(chan control),
		stopChan: make(chan struct{}),
	}
}

func (d *DB) FindFromNodeToKind(fromName, fromNamespace, fromKind string, toKind string, filter []string) []Resource {
	var resList []Resource
	srcNode := graph.Node{Name: fromName, Namespace: fromNamespace, Kind: fromKind}
	dstNode := &graph.Node{Kind: toKind}
	d.graph.TraverseFrom(srcNode, dstNode, func(n *graph.Node) {
		if n.Kind == dstNode.Kind {
			resList = append(resList, &res{
				Name:      n.Name,
				Kind:      n.Kind,
				Namespace: n.Namespace,
			})
		}
	}, filter...)
	return resList
}

/*
func (d *DB) Search(from graph.Node, to *graph.Node, filter []string) []Resource {
	var resList []Resource
	d.graph.TraverseFrom(from, to, func(n *graph.Node) {
		if n.Kind == to.Kind {
			resList = append(resList, &res{
				Name:      n.Name,
				Kind:      n.Kind,
				Namespace: n.Namespace,
			})
		}
	}, filter...)
	return resList
}
*/

func (d *DB) Search(from graph.Node, to *graph.Node, filter []string) []*graph.Node {
	var nodeList []*graph.Node
	d.graph.TraverseFrom(from, to, func(n *graph.Node) {
		if n.Kind == to.Kind {
			nodeList = append(nodeList, n)
		}
	}, filter...)
	return nodeList
}

type res struct {
	Name      string
	Namespace string
	Kind      string
}

func (r *res) GetName() string {
	return r.Name
}

func (r *res) GetNamespace() string {
	return r.Namespace
}

func (r *res) GetKind() string {
	return r.Kind
}

func (r *res) GetReferences() [][]string {
	return [][]string{}
}

type Resource interface {
	GetName() string
	GetNamespace() string
	GetKind() string
	GetReferences() [][]string
}

func (d *DB) InitNodes(items []Resource) {
	for _, item := range items {
		d.graph.AddNode(graph.Node{Name: item.GetName(), Namespace: item.GetNamespace(), Kind: item.GetKind()})
		klog.Infof("added %s node %s/%s", item.GetKind(), item.GetNamespace(), item.GetName())
	}
}

func (d *DB) InitEdges(items []Resource) {
	for _, item := range items {
		srcNode, ok := d.graph.GetNode(graph.Node{Name: item.GetName(), Namespace: item.GetNamespace(), Kind: item.GetKind()})
		if !ok {
			continue
		}
		referenceList := item.GetReferences()
		for _, ref := range referenceList {
			if dstNode, ok := d.graph.GetNode(graph.Node{Name: ref[0], Namespace: ref[1], Kind: ref[2]}); ok {
				d.graph.AddEdge(srcNode, dstNode)
				klog.Infof("added edge from %s %s to %s %s", srcNode.Kind, srcNode.Name, dstNode.Kind, dstNode.Name)
			}
		}
	}
}
func (d *DB) Start() error {
	go d.run()
	<-d.stopChan
	return nil
}

func (d *DB) Add(item Resource) {
	ctrl := control{
		action:     add,
		kind:       item.GetKind(),
		namespace:  item.GetNamespace(),
		name:       item.GetName(),
		references: item.GetReferences(),
	}
	d.ctrlChan <- ctrl
}

func (d *DB) Del(item Resource) {
	ctrl := control{
		action:    del,
		kind:      item.GetKind(),
		namespace: item.GetNamespace(),
		name:      item.GetName(),
	}
	d.ctrlChan <- ctrl
}

func (d *DB) run() {
	for ctrl := range d.ctrlChan {
		switch ctrl.action {
		case add:
			srcNode, ok := d.graph.GetNode(graph.Node{Name: ctrl.name, Namespace: ctrl.namespace, Kind: ctrl.kind})
			if !ok {
				d.graph.AddNode(graph.Node{Name: ctrl.name, Namespace: ctrl.namespace, Kind: ctrl.kind})
				srcNode, _ = d.graph.GetNode(graph.Node{Name: ctrl.name, Namespace: ctrl.namespace, Kind: ctrl.kind})
			}
			referenceList := ctrl.references
			for _, ref := range referenceList {
				if dstNode, ok := d.graph.GetNode(graph.Node{Name: ref[0], Namespace: ref[1], Kind: ref[2]}); ok {
					d.graph.AddEdge(srcNode, dstNode)
					klog.Infof("added edge from %s %s to %s %s", srcNode.Kind, srcNode.Name, dstNode.Kind, dstNode.Name)
				}
			}
		case del:
			d.graph.DelEdge(&graph.Node{Name: ctrl.name, Namespace: ctrl.namespace, Kind: ctrl.kind})
			d.graph.DelNode(graph.Node{Name: ctrl.name, Namespace: ctrl.namespace, Kind: ctrl.kind})
		}
	}
}
