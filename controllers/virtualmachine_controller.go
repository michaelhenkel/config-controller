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
	"k8s.io/apimachinery/pkg/api/errors"
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

type VirtualMachine struct {
	*contrail.VirtualMachine
}

func (res *VirtualMachine) GetName() string {
	return res.Name
}

func (res *VirtualMachine) GetNamespace() string {
	return res.Namespace
}

func (res *VirtualMachine) GetKind() string {
	return res.Kind
}

func (res *VirtualMachine) GetReferences() [][]string {
	return [][]string{}
}

func init() {
	ControllerMap["VirtualMachine"] = &VirtualMachineReconciler{}
}

func (r *VirtualMachineReconciler) New(client client.Client, contrailClient *contrailClientset.Clientset, scheme *runtime.Scheme) ResourceController {
	return &VirtualMachineReconciler{
		Client:         client,
		Scheme:         scheme,
		contrailClient: contrailClient,
	}
}

func (r *VirtualMachineReconciler) InitNodes() ([]db.Resource, error) {
	resourceList, err := r.contrailClient.CoreV1alpha1().VirtualMachines().List(context.Background(), metav1.ListOptions{})
	if err != nil {
		klog.Error(err)
		return nil, err
	}
	var objList []db.Resource
	for _, r := range resourceList.Items {
		var dbResource db.Resource
		obj := &VirtualMachine{
			VirtualMachine: &r,
		}
		dbResource = obj
		objList = append(objList, dbResource)
	}
	return objList, nil
}

// ConfigReconciler reconciles a Config object
type VirtualMachineReconciler struct {
	client.Client
	Scheme         *runtime.Scheme
	contrailClient *contrailClientset.Clientset
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
func (r *VirtualMachineReconciler) Reconcile(ctx context.Context, req ctrl.Request) (ctrl.Result, error) {
	_ = log.FromContext(ctx)

	res := &contrail.VirtualMachine{}

	if err := r.Client.Get(ctx, types.NamespacedName{Name: req.Name, Namespace: req.Namespace}, res); err != nil {
		if errors.IsNotFound(err) {
			klog.Info("resource not found")
			return ctrl.Result{}, nil
		} else {
			klog.Error(err)
			return ctrl.Result{}, err
		}
	}
	klog.Infof("got %s %s/%s", res.Kind, res.Namespace, res.Name)

	return ctrl.Result{}, nil
}

// SetupWithManager sets up the controller with the Manager.
func (r *VirtualMachineReconciler) SetupWithManager(mgr ctrl.Manager) error {
	return ctrl.NewControllerManagedBy(mgr).
		For(&contrail.VirtualMachine{}).
		Complete(r)
}
