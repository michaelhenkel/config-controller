package graph

import (
	"sync"
)

// Node a single node that composes the tree
type Node struct {
	Name      string
	Namespace string
	Kind      string
}

// ItemGraph the Items graph
type ItemGraph struct {
	nodes map[Node]*Node
	edges map[Node]map[Node]*Node
	lock  sync.RWMutex
}

// AddNode adds a node to the graph
func (g *ItemGraph) AddNode(n Node) {
	g.lock.Lock()
	if g.nodes == nil {
		g.nodes = make(map[Node]*Node)
	}
	g.nodes[n] = &n
	g.lock.Unlock()
}

// AddNode adds a node to the graph
func (g *ItemGraph) GetNode(node Node) (*Node, bool) {
	g.lock.Lock()
	n, ok := g.nodes[node]
	g.lock.Unlock()
	return n, ok
}

// AddNode adds a node to the graph
func (g *ItemGraph) DelNode(node Node) {
	g.lock.Lock()
	delete(g.nodes, node)
	g.lock.Unlock()
}

// AddEdge adds an edge to the graph
func (g *ItemGraph) EdgeExists(n1, n2 *Node) bool {
	node1_2 := false
	node2_1 := false
	g.lock.Lock()
	if g.edges != nil {
		if edge, ok := g.edges[*n1]; ok {
			if _, ok := edge[*n2]; ok {
				node1_2 = true
			}
		}
		if edge, ok := g.edges[*n2]; ok {
			if _, ok := edge[*n1]; ok {
				node2_1 = true
			}
		}
	}
	g.lock.Unlock()
	return node1_2 && node2_1
}

// AddEdge adds an edge to the graph
func (g *ItemGraph) DelEdge(node *Node) {
	g.lock.Lock()
	if g.edges != nil {
		edges := g.edges[*node]
		for n, _ := range edges {
			if edge, ok := g.edges[n]; ok {
				delete(edge, *node)
			}
		}
		delete(g.edges, *node)
	}
	g.lock.Unlock()
}

// AddEdge adds an edge to the graph
func (g *ItemGraph) AddEdge(n1, n2 *Node) {
	g.lock.Lock()
	if g.edges == nil {
		g.edges = make(map[Node]map[Node]*Node)
	}
	if edge, ok := g.edges[*n1]; !ok {
		g.edges[*n1] = map[Node]*Node{*n2: n2}
	} else {
		edge[*n2] = n2
	}
	if edge, ok := g.edges[*n2]; !ok {
		g.edges[*n2] = map[Node]*Node{*n1: n1}
	} else {
		edge[*n1] = n1
	}
	g.lock.Unlock()
}

func (g *ItemGraph) TraverseFrom(from Node, to *Node, f func(*Node), filterList ...string) {
	var filterMap = make(map[string]struct{})
	var filter bool
	if len(filterList) > 0 {
		filter = true
		for _, filter := range filterList {
			filterMap[filter] = struct{}{}
		}
	}
	g.lock.RLock()
	q := NodeQueue{}
	q.New()
	var n *Node
	n, ok := g.nodes[from]
	if !ok {
		g.lock.RUnlock()
		return
	}
	q.Enqueue(*n)
	visited := make(map[Node]bool)
	for {
		if q.IsEmpty() {
			break
		}
		node := q.Dequeue()
		visited[*node] = true
		if node.Kind != to.Kind {
			near := g.edges[*node]
			for _, j := range near {
				ignore := false
				if filter {
					if _, ok := filterMap[j.Kind]; !ok {
						ignore = true
					}
				}
				if !visited[*j] && !ignore {
					q.Enqueue(*j)
					visited[*j] = true
				}
			}
		}
		if f != nil {
			f(node)
		}
	}
	g.lock.RUnlock()
}
