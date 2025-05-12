"use client";

import { useEffect, useState, useRef } from "react";
import { useRouter } from "next/navigation";
import {
  Box,
  Button,
  Container,
  Divider,
  Grid,
  List,
  ListItem,
  ListItemText,
  Paper,
  TextField,
  Typography,
  Drawer,
  IconButton,
  ListItemButton,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
} from "@mui/material";
import MenuIcon from "@mui/icons-material/Menu";
import CloseIcon from "@mui/icons-material/Close";
import AddIcon from "@mui/icons-material/Add";
import SendIcon from "@mui/icons-material/Send";
import PeopleIcon from "@mui/icons-material/People";
import ExitToAppIcon from "@mui/icons-material/ExitToApp";
import useMediaQuery from "@mui/material/useMediaQuery";
import { useTheme } from "@mui/material/styles";

interface ClientMessage {
  type: string;
  [key: string]: any;
}

interface ServerMessage {
  type: string;
  [key: string]: any;
}

interface ChatMessage {
  sender: string;
  content: string;
  room_name: string;
  timestamp: string;
}

export default function page() {
  const [username, setUsername] = useState<string>("");
  const [userId, setUserId] = useState<string>("");
  const [socket, setSocket] = useState<WebSocket | null>(null);
  const [connected, setConnected] = useState<boolean>(false);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [messageInput, setMessageInput] = useState<string>("");
  const [rooms, setRooms] = useState<string[]>(["general"]);
  const [currentRoom, setCurrentRoom] = useState<string>("general");
  const [users, setUsers] = useState<string[]>([]);
  const [drawerOpen, setDrawerOpen] = useState<boolean>(false);
  const [createRoomDialogOpen, setCreateRoomDialogOpen] =
    useState<boolean>(false);
  const [newRoomName, setNewRoomName] = useState<string>("");
  const [usersDrawerOpen, setUsersDrawerOpen] = useState<boolean>(false);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const router = useRouter();
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down("md"));

  useEffect(() => {
    const storedUsername = localStorage.getItem("chat_username");
    if (!storedUsername) {
      router.push("/");
      return;
    }

    setUsername(storedUsername);
    connectWebSocket(storedUsername);

    return () => {
      // コンポーネントの案マウント時にWebSocket接続を閉じる
      if (socket) {
        socket.close();
      }
    };
  }, []);

  // メッセージが追加されたときに自動スクロール
  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  const connectWebSocket = (username: string) => {
    const ws = new WebSocket("ws://localhost:8080/ws");

    ws.onopen = () => {
      setConnected(true);

      const loginMessage: ClientMessage = {
        type: "Login",
        username: username,
      };
      ws.send(JSON.stringify(loginMessage));
    };

    ws.onmessage = (event) => {
      const message: ServerMessage = JSON.parse(event.data);
      handleServerMessage(message);
    };

    ws.onclose = () => {
      setConnected(false);
    };

    ws.onerror = (error) => {
      console.error("WebSocket error: ", error);
    };

    setSocket(ws);
  };

  // サーバーからのメッセージ処理
  const handleServerMessage = (message: ServerMessage) => {
    console.log("Received message:", message);

    switch (message.type) {
      case "Welcome":
        setUserId(message.user_id);
        // ルーム一覧を取得
        sendMessage({ type: "ListRooms" });
        // 現在のルームのユーザー一覧を取得
        sendMessage({ type: "ListUsers" });
        break;

      case "NewMessage":
        if (message.room_name === currentRoom) {
          // 現在表示中のルームにメッセージが送信された場合、表示する
          setMessages((prevMessages) => [
            ...prevMessages,
            {
              sender: message.sender,
              content: message.content,
              room_name: message.room_name,
              timestamp: message.timestamp,
            },
          ]);
        }
        break;

      case "UserJoined":
        // ユーザー一覧を更新するためにリクエスト
        if (message.room_name === currentRoom) {
          sendMessage({ type: "ListUsers" });
        }
        break;

      case "UserLeft":
        // ユーザー一覧を更新するためにリクエスト
        if (message.room_name === currentRoom) {
          sendMessage({ type: "ListUsers" });
        }
        break;

      case "RoomCreated":
        // ルーム一覧を更新
        sendMessage({ type: "ListRooms" });
        break;

      case "JoinedRoom":
        // ルーム変更時の処理
        setCurrentRoom(message.room_name);
        setMessages([]); // メッセージをクリア
        // ユーザー一覧を取得
        sendMessage({ type: "ListUsers" });
        setDrawerOpen(false); // モバイルの場合、ドロワーを閉じる
        break;

      case "RoomList":
        setRooms(message.rooms);
        break;

      case "UserList":
        setUsers(message.users);
        break;

      case "Error":
        console.error("Server error:", message.message);
        // エラーメッセージを表示する処理を追加できます
        break;

      default:
        break;
    }
  };

  const sendMessage = (message: ClientMessage) => {
    if (socket && connected) {
      socket.send(JSON.stringify(message));
    }
  };

  const handleSendMessage = (e: React.FormEvent) => {
    e.preventDefault();
    if (messageInput.trim() && connected) {
      const message: ClientMessage = {
        type: "SendMessage",
        content: messageInput,
      };
      sendMessage(message);
      setMessageInput("");
    }
  };

  const handleRoomChange = (roomName: string) => {
    if (roomName !== currentRoom) {
      const message: ClientMessage = {
        type: "JoinRoom",
        room_name: roomName,
      };
      sendMessage(message);
    }
  };

  const handleCreateRoom = () => {
    if (newRoomName.trim() && connected) {
      const message: ClientMessage = {
        type: "CreateRoom",
        room_name: newRoomName,
      };
      sendMessage(message);
      setNewRoomName("");
      setCreateRoomDialogOpen(false);
    }
  };

  const handleLogout = () => {
    localStorage.removeItem("chat_username");
    if (socket) {
      socket.close();
    }
    router.push("/");
  };

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString();
  };

  const drawerContent = (
    <Box sx={{ width: 250, p: 2 }}>
      <Box
        sx={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          mb: 2,
        }}
      >
        <Typography variant="h6">ルーム一覧</Typography>
        {isMobile && (
          <IconButton onClick={() => setDrawerOpen(false)}>
            <CloseIcon />
          </IconButton>
        )}
      </Box>
      <Divider sx={{ mb: 2 }} />
      <List>
        {rooms.map((room) => (
          <ListItemButton
            key={room}
            selected={currentRoom === room}
            onClick={() => handleRoomChange(room)}
          >
            <ListItemText primary={room} />
          </ListItemButton>
        ))}
      </List>
      <Button
        variant="outlined"
        startIcon={<AddIcon />}
        onClick={() => setCreateRoomDialogOpen(true)}
        fullWidth
        sx={{ mt: 2 }}
      >
        新しいルーム
      </Button>
    </Box>
  );

  const usersDrawerContent = (
    <Box sx={{ width: 250, p: 2 }}>
      <Box
        sx={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          mb: 2,
        }}
      >
        <Typography variant="h6">ユーザー一覧</Typography>
        <IconButton onClick={() => setUsersDrawerOpen(false)}>
          <CloseIcon />
        </IconButton>
      </Box>
      <Divider sx={{ mb: 2 }} />
      <List>
        {users.map((user) => (
          <ListItem key={user}>
            <ListItemText primary={user} />
          </ListItem>
        ))}
      </List>
    </Box>
  );

  return (
    <Container maxWidth="xl" sx={{ height: "100vh", py: 2 }}>
      <Box sx={{ display: "flex", height: "100%" }}>
        {/* サイドドロワー（モバイル） */}
        {isMobile && (
          <Drawer
            anchor="left"
            open={drawerOpen}
            onClose={() => setDrawerOpen(false)}
          >
            {drawerContent}
          </Drawer>
        )}

        {/* ユーザードロワー（モバイル） */}
        {isMobile && (
          <Drawer
            anchor="right"
            open={usersDrawerOpen}
            onClose={() => setUsersDrawerOpen(false)}
          >
            {usersDrawerContent}
          </Drawer>
        )}

        {/* サイドバー（デスクトップ） */}
        {!isMobile && (
          <Box
            sx={{
              width: 250,
              flexShrink: 0,
              borderRight: 1,
              borderColor: "divider",
              display: { xs: "none", md: "block" },
            }}
          >
            {drawerContent}
          </Box>
        )}

        {/* メインコンテンツ */}
        <Box sx={{ flexGrow: 1, display: "flex", flexDirection: "column" }}>
          {/* ヘッダー */}
          <Paper
            elevation={2}
            sx={{
              p: 2,
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              mb: 2,
            }}
          >
            <Box sx={{ display: "flex", alignItems: "center" }}>
              {isMobile && (
                <IconButton
                  edge="start"
                  color="inherit"
                  onClick={() => setDrawerOpen(true)}
                  sx={{ mr: 2 }}
                >
                  <MenuIcon />
                </IconButton>
              )}
              <Typography variant="h6">{currentRoom}</Typography>
            </Box>
            <Box sx={{ display: "flex", alignItems: "center" }}>
              <Button
                variant="text"
                startIcon={<PeopleIcon />}
                onClick={() => setUsersDrawerOpen(true)}
                sx={{ mr: 1, display: { xs: "flex", md: "none" } }}
              >
                ユーザー
              </Button>
              <Button
                variant="text"
                startIcon={<ExitToAppIcon />}
                onClick={handleLogout}
              >
                ログアウト
              </Button>
            </Box>
          </Paper>

          {/* メッセージリスト */}
          <Paper
            elevation={1}
            sx={{
              p: 2,
              flexGrow: 1,
              overflow: "auto",
              mb: 2,
              display: "flex",
              flexDirection: "column",
            }}
          >
            <Box sx={{ flexGrow: 1, overflow: "auto" }}>
              {messages.map((msg, index) => (
                <Box
                  key={index}
                  sx={{
                    display: "flex",
                    flexDirection: "column",
                    alignItems:
                      msg.sender === username ? "flex-end" : "flex-start",
                    mb: 2,
                  }}
                >
                  <Box
                    sx={{
                      backgroundColor:
                        msg.sender === username ? "primary.main" : "grey.200",
                      color: msg.sender === username ? "white" : "text.primary",
                      borderRadius: 2,
                      px: 2,
                      py: 1,
                      maxWidth: "70%",
                    }}
                  >
                    <Typography variant="body1">{msg.content}</Typography>
                  </Box>
                  <Typography variant="caption" sx={{ mt: 0.5 }}>
                    {msg.sender === username ? "あなた" : msg.sender} •{" "}
                    {formatTimestamp(msg.timestamp)}
                  </Typography>
                </Box>
              ))}
              <div ref={messagesEndRef} />
            </Box>
          </Paper>

          {/* メッセージ入力フォーム */}
          <Paper elevation={2} sx={{ p: 2 }}>
            <form onSubmit={handleSendMessage}>
              <Box sx={{ display: "flex", gap: 2 }}>
                <Box sx={{ flexGrow: 1 }}>
                  <TextField
                    fullWidth
                    placeholder="メッセージを入力..."
                    value={messageInput}
                    onChange={(e) => setMessageInput(e.target.value)}
                    disabled={!connected}
                  />
                </Box>
                <Box>
                  <Button
                    type="submit"
                    variant="contained"
                    endIcon={<SendIcon />}
                    disabled={!messageInput.trim() || !connected}
                  >
                    送信
                  </Button>
                </Box>
              </Box>
            </form>
          </Paper>
        </Box>

        {/* ユーザーリスト（デスクトップ） */}
        {!isMobile && (
          <Box
            sx={{
              width: 250,
              flexShrink: 0,
              borderLeft: 1,
              borderColor: "divider",
              p: 2,
              display: { xs: "none", md: "block" },
            }}
          >
            <Typography variant="h6" sx={{ mb: 2 }}>
              ユーザー一覧
            </Typography>
            <Divider sx={{ mb: 2 }} />
            <List>
              {users.map((user) => (
                <ListItem key={user}>
                  <ListItemText primary={user} />
                </ListItem>
              ))}
            </List>
          </Box>
        )}
      </Box>

      {/* 新しいルーム作成ダイアログ */}
      <Dialog
        open={createRoomDialogOpen}
        onClose={() => setCreateRoomDialogOpen(false)}
      >
        <DialogTitle>新しいルームを作成</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="ルーム名"
            fullWidth
            value={newRoomName}
            onChange={(e) => setNewRoomName(e.target.value)}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateRoomDialogOpen(false)}>
            キャンセル
          </Button>
          <Button onClick={handleCreateRoom} disabled={!newRoomName.trim()}>
            作成
          </Button>
        </DialogActions>
      </Dialog>
    </Container>
  );
}
