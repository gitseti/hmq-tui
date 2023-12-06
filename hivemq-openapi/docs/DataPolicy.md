# DataPolicy

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | Option<**String**> | The formatted UTC timestamp indicating when the policy was created. | [optional][readonly]
**id** | **String** | The unique identifier of the policy. | 
**last_updated_at** | Option<**String**> | The formatted UTC timestamp indicating when the policy was updated the last time. | [optional][readonly]
**matching** | [**crate::models::DataPolicyMatching**](DataPolicyMatching.md) |  | 
**on_failure** | Option<[**crate::models::DataPolicyAction**](DataPolicyAction.md)> |  | [optional]
**on_success** | Option<[**crate::models::DataPolicyAction**](DataPolicyAction.md)> |  | [optional]
**validation** | Option<[**crate::models::DataPolicyValidation**](DataPolicyValidation.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


