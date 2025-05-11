document.addEventListener("DOMContentLoaded", () => {
  // DOM要素
  const loginPanel = document.getElementById("loginPanel");
  const chatContainer = document.getElementById("chatContainer");
  const usernameInput = document.getElementById("usernameInput");
  const loginButton = document.getElementById("loginButton");
  const messageContainer = document.getElementById("messageContainer");
  const messageInput = document.getElementById("messageInput");
  const sendButton = document.getElementById("sendButton");
  const roomList = document.getElementById("roomList");
  const userList = document.getElementById("userList");
  const currentRoomHeader = document.getElementById("currentRoom");
  const createRoomButton = document.getElementById("createRoomButton");
  const createRoomModal = document.getElementById("createRoomModal");
  const roomNameInput = document.getElementById("roomNameInput");
  const cancelCreateRoom = document.getElementById("cancelCreateRoom");
  const confirmCreateRoom = document.getElementById("confirmCreateRoom");

  // WebSocket接続
  let socket = null;
  let currentRoom = "general";
  let currentUsername = "";
  let currentUserId = "";

  // ログインボタンクリック
  loginButton.addEventListener("click", () => {
    const username = usernameInput.value.trim();
    if (username) {
      connectWebSocket(username);
    }
  });

  // Enterキーでログイン
  usernameInput.addEventListener("keypress", (e) => {
    if (e.key === "Enter") {
      const username = usernameInput.value.trim();
      if (username) {
        connectWebSocket(username);
      }
    }
  });

  // WebSocket接続
  function connectWebSocket(username) {
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const wsUrl = `${protocol}//${window.location.host}/ws`;

    socket = new WebSocket(wsUrl);

    socket.onopen = () => {
      console.log("WebSocket connected");

      // ログインメッセージを送信
      const loginMessage = {
        type: "Login",
        username: username,
      };
      socket.send(JSON.stringify(loginMessage));

      currentUsername = username;
    };

    socket.onmessage = (event) => {
      const message = JSON.parse(event.data);
      handleServerMessage(message);
    };

    socket.onclose = () => {
      console.log("WebSocket disconnected");
      // 再接続ロジックをここに追加することもできます
    };

    socket.onerror = (error) => {
      console.error("WebSocket error:", error);
    };
  }

  // サーバーからのメッセージを処理
  function handleServerMessage(message) {
    console.log("Received message:", message);

    switch (message.type) {
      case "Welcome":
        currentUserId = message.user_id;
        loginPanel.style.display = "none";
        chatContainer.style.display = "flex";

        // ルーム一覧を取得
        sendMessage({
          type: "ListRooms",
        });

        // ユーザー一覧を取得
        sendMessage({
          type: "ListUsers",
        });
        break;

      case "NewMessage":
        addChatMessage(message.sender, message.content, message.room_name);
        break;

      case "UserJoined":
        if (message.room_name === currentRoom) {
          addSystemMessage(`${message.username} がルームに参加しました`);

          // ユーザー一覧更新
          sendMessage({
            type: "ListUsers",
          });
        }
        break;

      case "UserLeft":
        if (message.room_name === currentRoom) {
          addSystemMessage(`${message.username} がルームを退出しました`);

          // ユーザー一覧更新
          sendMessage({
            type: "ListUsers",
          });
        }
        break;

      case "RoomList":
        updateRoomList(message.rooms);
        break;

      case "UserList":
        updateUserList(message.users);
        break;

      case "RoomCreated":
        addSystemMessage(`ルーム「${message.room_name}」が作成されました`);

        // ルーム一覧を更新
        sendMessage({
          type: "ListRooms",
        });

        // 作成したルームに参加
        sendMessage({
          type: "JoinRoom",
          room_name: message.room_name,
        });
        break;

      case "JoinedRoom":
        currentRoom = message.room_name;
        currentRoomHeader.textContent = currentRoom;

        // ルームリストのアクティブ項目を更新
        const roomItems = roomList.querySelectorAll("li");
        roomItems.forEach((item) => {
          if (item.dataset.room === currentRoom) {
            item.classList.add("active");
          } else {
            item.classList.remove("active");
          }
        });

        // メッセージをクリア
        messageContainer.innerHTML = "";

        addSystemMessage(`「${currentRoom}」に参加しました`);

        // ユーザー一覧を更新
        sendMessage({
          type: "ListUsers",
        });
        break;

      case "Error":
        addSystemMessage(`エラー: ${message.message}`);
        break;
    }
  }

  // メッセージ送信ボタンクリック
  sendButton.addEventListener("click", () => {
    sendChatMessage();
  });

  // Enterキーでメッセージ送信
  messageInput.addEventListener("keypress", (e) => {
    if (e.key === "Enter") {
      sendChatMessage();
    }
  });

  // チャットメッセージ送信
  function sendChatMessage() {
    const content = messageInput.value.trim();
    if (content && socket) {
      sendMessage({
        type: "SendMessage",
        content: content,
      });

      messageInput.value = "";
    }
  }

  // WebSocketを通じてメッセージを送信する汎用関数
  function sendMessage(message) {
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify(message));
    }
  }

  // チャットメッセージをUIに追加
  function addChatMessage(sender, content, roomName) {
    if (roomName !== currentRoom) return;

    const messageElement = document.createElement("div");
    messageElement.className = `message ${
      sender === currentUsername ? "sent" : "received"
    }`;

    const usernameElement = document.createElement("div");
    usernameElement.className = "username";
    usernameElement.textContent = sender;

    const contentElement = document.createElement("div");
    contentElement.className = "content";
    contentElement.textContent = content;

    messageElement.appendChild(usernameElement);
    messageElement.appendChild(contentElement);

    messageContainer.appendChild(messageElement);
    scrollToBottom();
  }

  // システムメッセージをUIに追加
  function addSystemMessage(text) {
    const messageElement = document.createElement("div");
    messageElement.className = "system-message";
    messageElement.textContent = text;

    messageContainer.appendChild(messageElement);
    scrollToBottom();
  }

  // メッセージコンテナを最下部にスクロール
  function scrollToBottom() {
    messageContainer.scrollTop = messageContainer.scrollHeight;
  }

  // ルームリストを更新
  function updateRoomList(rooms) {
    roomList.innerHTML = "";

    rooms.forEach((room) => {
      const roomElement = document.createElement("li");
      roomElement.textContent = room;
      roomElement.dataset.room = room;

      if (room === currentRoom) {
        roomElement.classList.add("active");
      }

      roomElement.addEventListener("click", () => {
        if (room !== currentRoom) {
          joinRoom(room);
        }
      });

      roomList.appendChild(roomElement);
    });
  }

  // ユーザーリストを更新
  function updateUserList(users) {
    userList.innerHTML = "";

    users.forEach((username) => {
      const userElement = document.createElement("li");
      userElement.textContent = username;

      if (username === currentUsername) {
        userElement.style.fontWeight = "bold";
      }

      userList.appendChild(userElement);
    });
  }

  // ルームに参加
  function joinRoom(roomName) {
    sendMessage({
      type: "JoinRoom",
      room_name: roomName,
    });
  }

  // 新規ルーム作成モーダル表示
  createRoomButton.addEventListener("click", () => {
    createRoomModal.style.display = "flex";
    roomNameInput.value = "";
    roomNameInput.focus();
  });

  // モーダルのキャンセルボタン
  cancelCreateRoom.addEventListener("click", () => {
    createRoomModal.style.display = "none";
  });

  // モーダルの外側クリックで閉じる
  createRoomModal.addEventListener("click", (e) => {
    if (e.target === createRoomModal) {
      createRoomModal.style.display = "none";
    }
  });

  // 新規ルーム作成
  confirmCreateRoom.addEventListener("click", () => {
    const roomName = roomNameInput.value.trim();
    if (roomName) {
      sendMessage({
        type: "CreateRoom",
        room_name: roomName,
      });
      createRoomModal.style.display = "none";
    }
  });

  // Enter キーでルーム作成
  roomNameInput.addEventListener("keypress", (e) => {
    if (e.key === "Enter") {
      const roomName = roomNameInput.value.trim();
      if (roomName) {
        sendMessage({
          type: "CreateRoom",
          room_name: roomName,
        });
        createRoomModal.style.display = "none";
      }
    }
  });

  // 接続が切れた時の処理
  window.addEventListener("beforeunload", () => {
    if (socket) {
      socket.close();
    }
  });
});
