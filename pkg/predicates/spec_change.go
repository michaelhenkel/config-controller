package predicates

import (
	"k8s.io/apimachinery/pkg/api/equality"
	"k8s.io/apimachinery/pkg/runtime"
	"sigs.k8s.io/controller-runtime/pkg/event"
	"sigs.k8s.io/controller-runtime/pkg/predicate"

	contrail "ssd-git.juniper.net/contrail/cn2/contrail/pkg/apis/core/v1alpha1"
)

// PodIPChangedPredicate implements a default update predicate function on
// PodIP status state change.
type VirtualMachineInterfaceSpecChangePredicate struct {
	predicate.Funcs
	Scheme *runtime.Scheme
}

func (p VirtualMachineInterfaceSpecChangePredicate) Update(e event.UpdateEvent) bool {
	if e.ObjectOld == nil {
		return false
	}
	if e.ObjectNew == nil {
		return false
	}
	oldObj, ok := e.ObjectOld.(*contrail.VirtualMachineInterface)
	if !ok {
		return false
	}

	newObj, ok := e.ObjectNew.(*contrail.VirtualMachineInterface)
	if !ok {
		return false
	}

	specDiff := equality.Semantic.DeepDerivative(newObj.Spec, oldObj.Spec)
	//statusDiff := equality.Semantic.DeepDerivative(newObj.Status, oldObj.Status)
	//fmt.Println(cmp.Diff(newObj.Spec, oldObj.Spec))
	//fmt.Println(cmp.Diff(newObj.Status, oldObj.Status))
	//return specDiff || statusDiff
	return specDiff
}
