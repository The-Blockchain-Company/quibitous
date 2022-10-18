# REST Api

It is possible to query the node via its REST Interface.

In the node configuration, you have set something like:

```yaml
# ...

rest:
  listen: "127.0.0.1:8443"

#...
```

This is the REST endpoint to talk to the node, to query blocks or send transaction.

It is possible to query the node stats with the following end point:

```sh
curl http://127.0.0.1:8443/api/v0/node/stats
```

The result may be:

```json
{"blockRecvCnt":120,"txRecvCnt":92,"uptime":245}
```

> THE REST API IS STILL UNDER DEVELOPMENT

Please note that the end points and the results may change in the future.

To see the whole Node API documentation,
[click here](https://editor.swagger.io/?url=https://raw.githubusercontent.com/the-blockchain-company/quibitous/git+https://github.com/the-blockchain-company/chain-libs.git?branch=main#45b943be97f8bad0c90318a72cf23fc20d923d56"/doc/api/v0.yaml)
