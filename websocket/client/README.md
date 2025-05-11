# Output

```sh
Received from server: Response
```

## 実装ポイント

- `connect_async` でサーバーに接続します。 `ws://127.0.0.1:8080` はサーバーがリスンしている WebSocket の URL です。
- メッセージを送信後、サーバーからのレスポンスを受け取ります。
- `Message::Text` はテキストメッセージの送受信を扱うために使います。
