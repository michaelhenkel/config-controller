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
	"github.com/michaelhenkel/config-controller/pkg/predicates"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/runtime"
	"k8s.io/apimachinery/pkg/types"
	"k8s.io/klog/v2"
	ctrl "sigs.k8s.io/controller-runtime"
	"sigs.k8s.io/controller-runtime/pkg/builder"
	"sigs.k8s.io/controller-runtime/pkg/client"
	"sigs.k8s.io/controller-runtime/pkg/handler"
	"sigs.k8s.io/controller-runtime/pkg/log"
	"sigs.k8s.io/controller-runtime/pkg/source"
	contrail "ssd-git.juniper.net/contrail/cn2/contrail/pkg/apis/core/v1alpha1"
	contrailClientset "ssd-git.juniper.net/contrail/cn2/contrail/pkg/client/clientset_generated/clientset"
)

type VirtualMachineInterface struct {
	*contrail.VirtualMachineInterface
}

func (res *VirtualMachineInterface) GetName() string {
	return res.Name
}

func (res *VirtualMachineInterface) GetNamespace() string {
	return res.Namespace
}

func (res *VirtualMachineInterface) GetKind() string {
	return "VirtualMachineInterface"
}

func (res *VirtualMachineInterface) GetReferences() [][]string {
	var refList [][]string
	for _, ref := range res.Spec.VirtualMachineInterfaceReferences {
		refList = append(refList, []string{ref.Name, ref.Namespace, ref.Kind})
	}
	for _, ref := range res.Spec.VirtualMachineReferences {
		refList = append(refList, []string{ref.Name, ref.Namespace, ref.Kind})
	}
	refList = append(refList, []string{res.Spec.VirtualNetworkReference.Name, res.Spec.VirtualNetworkReference.Namespace, res.Spec.VirtualNetworkReference.Kind})
	return refList
}

func init() {
	ControllerMap["VirtualMachineInterface"] = &VirtualMachineInterfaceReconciler{}
}

func (res *VirtualMachineInterfaceReconciler) PathToNode() []string {
	return []string{"VirtualMachine", "VirtualRouter"}
}

func (res *VirtualMachineInterfaceReconciler) GetClient() client.Client {
	return res.Client
}

func (res *VirtualMachineInterfaceReconciler) GetDBClient() *db.DB {
	return res.dbClient
}

func (res *VirtualMachineInterfaceReconciler) GetChannel() chan NodeResource {
	return res.nodeResourceChan
}

func (r *VirtualMachineInterfaceReconciler) New(client client.Client, contrailClient *contrailClientset.Clientset, scheme *runtime.Scheme, dbClient *db.DB, nodeResourceChan chan NodeResource) ResourceController {
	return &VirtualMachineInterfaceReconciler{
		Client:           client,
		Scheme:           scheme,
		contrailClient:   contrailClient,
		dbClient:         dbClient,
		nodeResourceChan: nodeResourceChan,
	}
}

func (r *VirtualMachineInterfaceReconciler) InitNodes() ([]db.Resource, error) {
	resourceList, err := r.contrailClient.CoreV1alpha1().VirtualMachineInterfaces("").List(context.Background(), metav1.ListOptions{})
	if err != nil {
		klog.Error(err)
		return nil, err
	}
	var objList []db.Resource
	for idx := range resourceList.Items {
		res := resourceList.Items[idx]
		var dbResource db.Resource = &VirtualMachineInterface{
			VirtualMachineInterface: &res,
		}
		objList = append(objList, dbResource)
	}
	return objList, nil
}

// ConfigReconciler reconciles a Config object
type VirtualMachineInterfaceReconciler struct {
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
func (r *VirtualMachineInterfaceReconciler) Reconcile(ctx context.Context, req ctrl.Request) (ctrl.Result, error) {
	_ = log.FromContext(ctx)

	res := &contrail.VirtualMachineInterface{}

	if err := Process(ctx, r, req, &VirtualMachineInterface{
		VirtualMachineInterface: res,
	}, res); err != nil {
		klog.Error(err)
		return ctrl.Result{}, err
	}

	/*

		if err := r.Client.Get(ctx, types.NamespacedName{Name: req.Name, Namespace: req.Namespace}, res); err != nil {
			if errors.IsNotFound(err) {
				klog.Info("resource not found")
				SendToNode(res.Name, req.Namespace, res.Kind, []string{"VirtualMachine", "VirtualRouter"}, r.dbClient, r.nodeResourceChan)
				r.dbClient.Del(&VirtualMachineInterface{
					VirtualMachineInterface: res,
				})
				return ctrl.Result{}, nil
			} else {
				klog.Error(err)
				return ctrl.Result{}, err
			}
		}
		klog.Infof("got %s %s/%s", res.Kind, res.Namespace, res.Name)

		r.dbClient.Add(&VirtualMachineInterface{
			VirtualMachineInterface: res,
		})

		SendToNode(res.Name, req.Namespace, res.Kind, []string{"VirtualMachine", "VirtualRouter"}, r.dbClient, r.nodeResourceChan)
	*/

	return ctrl.Result{}, nil
}

// SetupWithManager sets up the controller with the Manager.
func (r *VirtualMachineInterfaceReconciler) SetupWithManager(mgr ctrl.Manager) error {
	return ctrl.NewControllerManagedBy(mgr).
		For(&contrail.VirtualMachineInterface{}).
		Watches(
			&source.Kind{Type: &contrail.VirtualMachineInterface{}},
			&handler.EnqueueRequestForObject{},
			builder.WithPredicates(predicates.VirtualMachineInterfaceSpecChangePredicate{}),
		).
		Complete(r)
}

func (r *VirtualMachineInterfaceReconciler) Get(name, namespace string) (interface{}, error) {
	res := &contrail.VirtualMachineInterface{}
	if err := r.Client.Get(context.Background(), types.NamespacedName{Name: name, Namespace: namespace}, res); err != nil {
		return nil, err
	}
	return res, nil
}
