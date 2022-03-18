package controllers

import (
	"context"

	pbv1 "github.com/michaelhenkel/config-controller/pkg/apis/v1"
	"github.com/michaelhenkel/config-controller/pkg/db"
	"k8s.io/apimachinery/pkg/api/errors"
	"k8s.io/apimachinery/pkg/runtime"
	"k8s.io/apimachinery/pkg/types"
	"k8s.io/klog/v2"
	ctrl "sigs.k8s.io/controller-runtime"
	"sigs.k8s.io/controller-runtime/pkg/client"
	contrailClientset "ssd-git.juniper.net/contrail/cn2/contrail/pkg/client/clientset_generated/clientset"
)

type ResourceController interface {
	SetupWithManager(mgr ctrl.Manager) error
	New(client client.Client, contrailClient *contrailClientset.Clientset, scheme *runtime.Scheme, dbClient *db.DB, nodeResourceChan chan NodeResource) ResourceController
	InitNodes() ([]db.Resource, error)
	Get(name, namespace string) (interface{}, error)
	PathToNode() []string
	GetClient() client.Client
	GetDBClient() *db.DB
	GetChannel() chan NodeResource
}

type NodeResource struct {
	*pbv1.Resource
	Node string
}

func FromNodeToResources(node string, kind string, filter []string, dbClient *db.DB) []db.Resource {
	return dbClient.FindFromNodeToKind(node, "", "VirtualRouter", kind, filter)
}

func FromResourceToNodes(name, namespace, kind string, filter []string, dbClient *db.DB) []db.Resource {
	return dbClient.FindFromNodeToKind(name, namespace, kind, "VirtualRouter", filter)
}

func SendToNode(name, namespace, kind string, filter []string, dbClient *db.DB, nodeResourceChan chan NodeResource) {
	for _, node := range dbClient.FindFromNodeToKind(name, namespace, kind, "VirtualRouter", filter) {
		klog.Infof("%s %s/%s -> %s", kind, namespace, name, node.GetName())
		nodeResource := &NodeResource{
			Resource: &pbv1.Resource{
				Name:      name,
				Namespace: namespace,
				Kind:      kind,
			},
			Node: node.GetName(),
		}
		nodeResourceChan <- *nodeResource
	}
}

func Process(ctx context.Context, resourceController ResourceController, req ctrl.Request, res db.Resource, obj client.Object) error {
	if err := resourceController.GetClient().Get(ctx, types.NamespacedName{Name: req.Name, Namespace: req.Namespace}, obj); err != nil {
		if errors.IsNotFound(err) {
			klog.Info("resource not found")
			SendToNode(req.Name, req.Namespace, res.GetKind(), resourceController.PathToNode(), resourceController.GetDBClient(), resourceController.GetChannel())
			resourceController.GetDBClient().Del(res)
			return nil
		} else {
			klog.Error(err)
			return err
		}
	}
	klog.Infof("got %s %s/%s", res.GetKind(), res.GetNamespace(), res.GetName())

	resourceController.GetDBClient().Add(res)

	SendToNode(res.GetName(), res.GetNamespace(), res.GetKind(), resourceController.PathToNode(), resourceController.GetDBClient(), resourceController.GetChannel())
	return nil
}

func AddUpdate() {

}

func Delete() {

}

var ControllerMap map[string]ResourceController

func init() {
	ControllerMap = make(map[string]ResourceController)
}
