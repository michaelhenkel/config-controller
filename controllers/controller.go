package controllers

import (
	"github.com/michaelhenkel/config-controller/pkg/db"
	"k8s.io/apimachinery/pkg/runtime"
	ctrl "sigs.k8s.io/controller-runtime"
	"sigs.k8s.io/controller-runtime/pkg/client"
	contrailClientset "ssd-git.juniper.net/contrail/cn2/contrail/pkg/client/clientset_generated/clientset"
)

type ResourceController interface {
	SetupWithManager(mgr ctrl.Manager) error
	New(client client.Client, contrailClient *contrailClientset.Clientset, scheme *runtime.Scheme) ResourceController
	InitNodes() ([]db.Resource, error)
}

type Resource interface {
}

var ControllerMap map[string]ResourceController

func init() {
	ControllerMap = make(map[string]ResourceController)
}
