package server

import (
	"context"
	"fmt"
	"net"

	"github.com/michaelhenkel/config-controller/controllers"
	pbv1 "github.com/michaelhenkel/config-controller/pkg/apis/v1"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"k8s.io/klog/v2"
	contrail "ssd-git.juniper.net/contrail/cn2/contrail/pkg/apis/core/v1alpha1"
)

type SubscriptionManager struct {
	Subscriptions     map[string]Subscription
	NewSubscriberChan chan string
}

func (sm *SubscriptionManager) AddSubscription(node string, subscription Subscription) {
	sm.Subscriptions[node] = subscription
}

func (sm *SubscriptionManager) RemoveSubscription(node string) {
	delete(sm.Subscriptions, node)
}

func (sm *SubscriptionManager) GetSubscriptionChannel(node string) chan pbv1.Resource {
	if subscription, ok := sm.Subscriptions[node]; ok {
		return subscription.Channel
	}
	return nil
}

func NewSubscriptionManager(newSubscriberChan chan string) *SubscriptionManager {
	var subscriptionMap = make(map[string]Subscription)
	return &SubscriptionManager{
		Subscriptions:     subscriptionMap,
		NewSubscriberChan: newSubscriberChan,
	}
}

type Subscription struct {
	Channel chan pbv1.Resource
	Init    bool
}

type ConfigController struct {
	pbv1.UnimplementedConfigControllerServer
	SubscriptionManager   *SubscriptionManager
	resourceControllerMap map[string]controllers.ResourceController
}

func New(subscriptionManager *SubscriptionManager, resourceControllerMap map[string]controllers.ResourceController) *ConfigController {
	s := &ConfigController{
		SubscriptionManager:   subscriptionManager,
		resourceControllerMap: resourceControllerMap,
	}
	return s
}

func (c *ConfigController) GetVirtualNetwork(ctx context.Context, res *pbv1.Resource) (*contrail.VirtualNetwork, error) {
	resource, err := c.resourceControllerMap[res.Kind].Get(res.Name, res.Namespace)
	if err != nil {

	}
	obj, ok := resource.(*contrail.VirtualNetwork)
	if ok {
		return obj, nil
	}
	return nil, fmt.Errorf("cannot decode to VirtualNetwork")
}

func (c *ConfigController) GetVirtualMachineInterface(ctx context.Context, res *pbv1.Resource) (*contrail.VirtualMachineInterface, error) {
	resource, err := c.resourceControllerMap[res.Kind].Get(res.Name, res.Namespace)
	if err != nil {

	}
	obj, ok := resource.(*contrail.VirtualMachineInterface)
	if ok {
		return obj, nil
	}
	return nil, fmt.Errorf("cannot decode to VirtualMachineInterface")
}

func (c *ConfigController) SubscribeListWatch(req *pbv1.SubscriptionRequest, srv pbv1.ConfigController_SubscribeListWatchServer) error {
	conn := make(chan pbv1.Resource)
	c.SubscriptionManager.AddSubscription(req.Name, Subscription{
		Channel: conn,
		Init:    false,
	})
	klog.Infof("new subscription request from node %s", req.Name)
	var stopChan = make(chan struct{})
	go func() {
		for {
			select {
			case <-srv.Context().Done():
				c.SubscriptionManager.RemoveSubscription(req.GetName())
				stopChan <- struct{}{}
			case response := <-conn:
				if status, ok := status.FromError(srv.Send(&response)); ok {
					switch status.Code() {
					case codes.OK:
					case codes.Unavailable, codes.Canceled, codes.DeadlineExceeded:
						stopChan <- struct{}{}
					default:
						stopChan <- struct{}{}
					}
				}
			}
		}
	}()
	klog.Info("sending new subscription msg")
	//c.k8sClient.NewSubscriber(req.Name, conn)
	<-stopChan
	return nil
}

func (c *Client) EventWatcher() {
	for nodeResource := range c.nodeResourceChan {
		if subscriber, ok := c.subscriptionManager.Subscriptions[nodeResource.Node]; ok {
			subscriber.Channel <- *nodeResource.Resource
		}
	}
}

func (c *Client) Start(resourceControllerMap map[string]controllers.ResourceController, nodeResourceChan chan controllers.NodeResource) error {
	var newSubscriberChan = make(chan string)
	subscriptionManager := NewSubscriptionManager(newSubscriberChan)
	c.subscriptionManager = subscriptionManager
	c.nodeResourceChan = nodeResourceChan
	grpcPort := 20443
	lis, err := net.Listen("tcp", fmt.Sprintf("0.0.0.0:%d", grpcPort))
	if err != nil {
		klog.Error(err, "unable to start grpc server")
		return err
	}
	var opts []grpc.ServerOption
	grpcServer := grpc.NewServer(opts...)
	s := New(subscriptionManager, resourceControllerMap)
	pbv1.RegisterConfigControllerServer(grpcServer, s)
	klog.Infof("starting GRPC server on port %d", grpcPort)
	//var watchStop = make(chan struct{})
	go c.EventWatcher()
	grpcServer.Serve(lis)
	return nil
}

type Client struct {
	subscriptionManager *SubscriptionManager
	nodeResourceChan    chan controllers.NodeResource
}

func NewClient() *Client {
	return &Client{}
}
