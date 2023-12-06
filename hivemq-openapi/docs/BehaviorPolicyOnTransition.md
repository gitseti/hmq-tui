# BehaviorPolicyOnTransition

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**connection_period_on_disconnect** | Option<[**crate::models::BehaviorPolicyOnEvent**](BehaviorPolicyOnEvent.md)> |  | [optional]
**event_period_on_any** | Option<[**crate::models::BehaviorPolicyOnEvent**](BehaviorPolicyOnEvent.md)> |  | [optional]
**mqtt_period_on_inbound_connect** | Option<[**crate::models::BehaviorPolicyOnEvent**](BehaviorPolicyOnEvent.md)> |  | [optional]
**mqtt_period_on_inbound_disconnect** | Option<[**crate::models::BehaviorPolicyOnEvent**](BehaviorPolicyOnEvent.md)> |  | [optional]
**mqtt_period_on_inbound_publish** | Option<[**crate::models::BehaviorPolicyOnEvent**](BehaviorPolicyOnEvent.md)> |  | [optional]
**mqtt_period_on_inbound_subscribe** | Option<[**crate::models::BehaviorPolicyOnEvent**](BehaviorPolicyOnEvent.md)> |  | [optional]
**from_state** | **String** | The exact state from which the transition happened. Alternatively a state filter can be used. | 
**to_state** | **String** | The exact state to which the transition happened. Alternatively a state filter can be used. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


