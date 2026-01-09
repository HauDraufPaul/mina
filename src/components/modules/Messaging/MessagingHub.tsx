import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import Modal from "@/components/ui/Modal";
import { Send, Plus, Paperclip, TrendingUp } from "lucide-react";
import { useErrorHandler } from "@/utils/errorHandler";
import { realtimeService } from "@/services/realtimeService";

interface MessagingConversation {
  id: number;
  name: string;
  created_at: number;
  updated_at: number;
}

interface Message {
  id: number;
  conversation_id: number;
  sender: string;
  content: string;
  created_at: number;
}

interface MessageAttachment {
  id: number;
  message_id: number;
  attachment_type: string;
  data_json: Record<string, unknown>;
  created_at: number;
}

export default function MessagingHub() {
  const [conversations, setConversations] = useState<MessagingConversation[]>([]);
  const [selectedConversation, setSelectedConversation] = useState<number | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [messageContent, setMessageContent] = useState("");
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newConversationName, setNewConversationName] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const errorHandler = useErrorHandler();

  useEffect(() => {
    loadConversations();
  }, []);

  useEffect(() => {
    if (selectedConversation) {
      loadMessages(selectedConversation);
    }
  }, [selectedConversation]);

  // Subscribe to real-time message updates
  useEffect(() => {
    const unsubscribe = realtimeService.subscribe("message", (data: Message) => {
      if (data.conversation_id === selectedConversation) {
        setMessages((prev) => {
          // Avoid duplicates
          if (prev.some((m) => m.id === data.id)) {
            return prev;
          }
          return [...prev, data].sort((a, b) => a.created_at - b.created_at);
        });
      }
    });
    return () => unsubscribe();
  }, [selectedConversation]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const loadConversations = async () => {
    try {
      setLoading(true);
      const result = await invoke<MessagingConversation[]>("messaging_list_conversations");
      setConversations(result);
      if (result.length > 0 && !selectedConversation) {
        setSelectedConversation(result[0].id);
      }
    } catch (err) {
      errorHandler.showError("Failed to load conversations", err);
    } finally {
      setLoading(false);
    }
  };

  const loadMessages = async (conversationId: number) => {
    try {
      const result = await invoke<Message[]>("get_conversation_messages", {
        conversationId,
        limit: 100,
      });
      setMessages(result);
    } catch (err) {
      errorHandler.showError("Failed to load messages", err);
    }
  };

  const createConversation = async () => {
    if (!newConversationName.trim()) {
      errorHandler.showError("Conversation name is required");
      return;
    }

    try {
      const id = await invoke<number>("messaging_create_conversation", { 
        name: newConversationName.trim() 
      });
      setNewConversationName("");
      setShowCreateModal(false);
      await loadConversations();
      setSelectedConversation(id);
      errorHandler.showSuccess("Conversation created");
    } catch (err) {
      errorHandler.showError("Failed to create conversation", err);
    }
  };

  const sendMessage = async () => {
    if (!selectedConversation || !messageContent.trim()) return;

    try {
      await invoke<number>("send_message", {
        conversationId: selectedConversation,
        sender: "user",
        content: messageContent.trim(),
      });

      setMessageContent("");
      // Don't reload - WebSocket will update in real-time
    } catch (err) {
      errorHandler.showError("Failed to send message", err);
    }
  };

  const attachTicker = async (ticker: string) => {
    if (!selectedConversation || !messageContent.trim()) return;

    try {
      const messageId = await invoke<number>("send_message", {
        conversationId: selectedConversation,
        sender: "user",
        content: messageContent.trim(),
      });

      await invoke<number>("attach_market_context", {
        messageId,
        attachmentType: "ticker",
        dataJson: { ticker },
      });

      setMessageContent("");
      // Don't reload - WebSocket will update in real-time
    } catch (err) {
      errorHandler.showError("Failed to attach ticker", err);
    }
  };

  return (
    <div className="flex h-[calc(100vh-200px)] gap-4">
      <div className="w-64 flex-shrink-0">
        <Card title="Conversations" subtitle={`${conversations.length} conversations`}>
          <div className="space-y-2">
            <Button variant="secondary" onClick={() => setShowCreateModal(true)} className="w-full">
              <Plus className="w-4 h-4 mr-2" />
              New Conversation
            </Button>
            <div className="space-y-1 max-h-[calc(100vh-300px)] overflow-y-auto">
              {conversations.map((conv) => (
                <button
                  key={conv.id}
                  onClick={() => setSelectedConversation(conv.id)}
                  className={`w-full text-left p-2 rounded border transition-colors ${
                    selectedConversation === conv.id
                      ? "border-neon-cyan bg-neon-cyan/10"
                      : "border-white/10 hover:border-white/20"
                  }`}
                >
                  <div className="font-semibold text-sm">{conv.name}</div>
                  <div className="text-xs text-gray-400">
                    {new Date(conv.updated_at * 1000).toLocaleDateString()}
                  </div>
                </button>
              ))}
            </div>
          </div>
        </Card>
      </div>

      <div className="flex-1 flex flex-col">
        {selectedConversation ? (
          <>
            <Card title="Messages" subtitle={`${messages.length} messages`} className="flex-1 flex flex-col">
              <div className="flex-1 overflow-y-auto space-y-4 p-4">
                {messages.map((message) => (
                  <div
                    key={message.id}
                    className={`flex ${message.sender === "user" ? "justify-end" : "justify-start"}`}
                  >
                    <div
                      className={`max-w-[70%] p-3 rounded ${
                        message.sender === "user"
                          ? "bg-neon-cyan/20 border border-neon-cyan/30"
                          : "bg-white/5 border border-white/10"
                      }`}
                    >
                      <div className="text-sm font-semibold mb-1">{message.sender}</div>
                      <div className="text-sm text-gray-200 whitespace-pre-wrap">{message.content}</div>
                      <div className="text-xs text-gray-400 mt-1">
                        {new Date(message.created_at * 1000).toLocaleTimeString()}
                      </div>
                    </div>
                  </div>
                ))}
                <div ref={messagesEndRef} />
              </div>

              <div className="border-t border-white/10 p-4">
                <div className="flex items-center gap-2">
                  <input
                    type="text"
                    value={messageContent}
                    onChange={(e) => setMessageContent(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" && !e.shiftKey) {
                        e.preventDefault();
                        sendMessage();
                      }
                    }}
                    placeholder="Type a message..."
                    className="flex-1 px-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan"
                  />
                  <Button variant="primary" onClick={sendMessage} disabled={!messageContent.trim()}>
                    <Send className="w-4 h-4" />
                  </Button>
                </div>
              </div>
            </Card>
          </>
        ) : (
          <Card title="No Conversation Selected" subtitle="Select or create a conversation">
            <div className="text-center py-12 text-gray-400">
              <p>Select a conversation from the sidebar or create a new one</p>
            </div>
          </Card>
        )}
      </div>

      <Modal
        isOpen={showCreateModal}
        onClose={() => {
          setShowCreateModal(false);
          setNewConversationName("");
        }}
        title="Create Conversation"
      >
        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-2">Conversation Name</label>
            <input
              type="text"
              value={newConversationName}
              onChange={(e) => setNewConversationName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  createConversation();
                }
              }}
              placeholder="Enter conversation name"
              className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan"
              autoFocus
            />
          </div>
          <div className="flex justify-end gap-2">
            <Button variant="secondary" onClick={() => {
              setShowCreateModal(false);
              setNewConversationName("");
            }}>
              Cancel
            </Button>
            <Button variant="primary" onClick={createConversation}>
              Create
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
