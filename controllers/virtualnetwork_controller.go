/*
Copyright 2022.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

package controllers

import (
	"context"

	"github.com/michaelhenkel/config-controller/pkg/db"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/runtime"
	"k8s.io/apimachinery/pkg/types"
	"k8s.io/klog/v2"
	ctrl "sigs.k8s.io/controller-runtime"
	"sigs.k8s.io/controller-runtime/pkg/client"
	"sigs.k8s.io/controller-runtime/pkg/log"
	contrail "ssd-git.juniper.net/contrail/cn2/contrail/pkg/apis/core/v1alpha1"
	contrailClientset "ssd-git.juniper.net/contrail/cn2/contrail/pkg/client/clientset_generated/clientset"
)

type VirtualNetwork struct {
	*contrail.VirtualNetwork
}

func (res *VirtualNetwork) GetName() string {
	return res.Name
}

func (res *VirtualNetwork) GetNamespace() string {
	return res.Namespace
}

func (res *VirtualNetwork) GetKind() string {
	return "VirtualNetwork"
}

func (res *VirtualNetwork) GetReferences() [][]string {
	var refList [][]string
	//refList = append(refList, []string{res.Spec.ProviderNetworkReference.Name, res.Spec.ProviderNetworkReference.Namespace, res.Spec.ProviderNetworkReference.Kind})
	//refList = append(refList, []string{res.Spec.V4SubnetReference.Name, res.Spec.V4SubnetReference.Namespace, res.Spec.V4SubnetReference.Kind})
	//refList = append(refList, []string{res.Spec.V6SubnetReference.Name, res.Spec.V6SubnetReference.Namespace, res.Spec.V6SubnetReference.Kind})
	return refList
}

func init() {
	ControllerMap["VirtualNetwork"] = &VirtualNetworkReconciler{}
}

func (res *VirtualNetworkReconciler) PathToNode() []string {
	return []string{"VirtualMachine", "VirtualMachineInterface", "VirtualRouter"}
}

func (res *VirtualNetworkReconciler) GetClient() client.Client {
	return res.Client
}

func (res *VirtualNetworkReconciler) GetDBClient() *db.DB {
	return res.dbClient
}

func (res *VirtualNetworkReconciler) GetChannel() chan NodeResource {
	return res.nodeResourceChan
}

func (r *VirtualNetworkReconciler) New(client client.Client, contrailClient *contrailClientset.Clientset, scheme *runtime.Scheme, dbClient *db.DB, nodeResourceChan chan NodeResource) ResourceController {
	return &VirtualNetworkReconciler{
		Client:           client,
		Scheme:           scheme,
		contrailClient:   contrailClient,
		dbClient:         dbClient,
		nodeResourceChan: nodeResourceChan,
	}
}

func (r *VirtualNetworkReconciler) InitNodes() ([]db.Resource, error) {
	resourceList, err := r.contrailClient.CoreV1alpha1().VirtualNetworks("").List(context.Background(), metav1.ListOptions{})
	if err != nil {
		klog.Error(err)
		return nil, err
	}
	var objList []db.Resource
	for idx := range resourceList.Items {
		res := resourceList.Items[idx]
		var dbResource db.Resource = &VirtualNetwork{
			VirtualNetwork: &res,
		}
		objList = append(objList, dbResource)
	}
	return objList, nil
}

// ConfigReconciler reconciles a Config object
type VirtualNetworkReconciler struct {
	client.Client
	Scheme           *runtime.Scheme
	contrailClient   *contrailClientset.Clientset
	dbClient         *db.DB
	nodeResourceChan chan NodeResource
}

//+kubebuilder:rbac:groups=core.contrail.juniper.net,resources=*,verbs=*

// Reconcile is part of the main kubernetes reconciliation loop which aims to
// move the current state of the cluster closer to the desired state.
// TODO(user): Modify the Reconcile function to compare the state specified by
// the Config object against the actual cluster state, and then
// perform operations to make the cluster state reflect the state specified by
// the user.
//
// For more details, check Reconcile and its Result here:
// - https://pkg.go.dev/sigs.k8s.io/controller-runtime@v0.11.0/pkg/reconcile
func (r *VirtualNetworkReconciler) Reconcile(ctx context.Context, req ctrl.Request) (ctrl.Result, error) {
	_ = log.FromContext(ctx)

	res := &contrail.VirtualNetwork{}

	if err := Process(ctx, r, req, &VirtualNetwork{
		VirtualNetwork: res,
	}, res); err != nil {
		klog.Error(err)
		return ctrl.Result{}, err
	}

	return ctrl.Result{}, nil
}

// SetupWithManager sets up the controller with the Manager.
func (r *VirtualNetworkReconciler) SetupWithManager(mgr ctrl.Manager) error {
	return ctrl.NewControllerManagedBy(mgr).
		For(&contrail.VirtualNetwork{}).
		Complete(r)
}

func (r *VirtualNetworkReconciler) Get(name, namespace string) (interface{}, error) {
	res := &contrail.VirtualNetwork{}
	if err := r.Client.Get(context.Background(), types.NamespacedName{Name: name, Namespace: namespace}, res); err != nil {
		return nil, err
	}
	return res, nil
}
