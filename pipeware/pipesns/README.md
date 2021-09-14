A [`pipebase`] plugin using [`aws-sdk-sns`] 
### Pipe Configs
| type | example |
| ---- | ------- |
| `SnsPublisherConfig` | [`sns_publisher`] |

### Error Handler Configs
| type | example |
| ---- | ------- |
| `SnsPipeErrorPublisherConfig` | [`sns_pipe_error_publisher`] |

[`pipebase`]: https://github.com/pipebase/pipebase
[`aws-sdk-sns`]: https://github.com/awslabs/aws-sdk-rust/tree/main/sdk/sns
[`sns_publisher`]: https://github.com/pipebase/pipebase/blob/main/examples/pubsub_sns/catalogs/sns_publisher.yml
[`sns_pipe_error_publisher`]: https://github.com/pipebase/pipebase/blob/main/examples/error_sns_publisher/catalogs/sns_pipe_error_publisher.yml