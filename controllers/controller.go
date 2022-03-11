package controllers

import (
	pbv1 "github.com/michaelhenkel/config-controller/pkg/apis/v1"
	"github.com/michaelhenkel/config-controller/pkg/db"
	"k8s.io/apimachinery/pkg/runtime"
	ctrl "sigs.k8s.io/controller-runtime"
	"sigs.k8s.io/controller-runtime/pkg/client"
	contrailClientset "ssd-git.juniper.net/contrail/cn2/contrail/pkg/client/clientset_generated/clientset"
)

type ResourceController interface {
	SetupWithManager(mgr ctrl.Manager) error
	New(client client.Client, contrailClient *contrailClientset.Clientset, scheme *runtime.Scheme, dbClient *db.DB, nodeResourceChan chan NodeResource) ResourceController
	InitNodes() ([]db.Resource, error)
	Get(name, namespace string) (interface{}, error)
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

var ControllerMap map[string]ResourceController

func init() {
	ControllerMap = make(map[string]ResourceController)
}
