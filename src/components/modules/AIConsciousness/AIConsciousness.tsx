import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import Modal from "../../ui/Modal";
import { MessageSquare, Plus, Send, Bot, User, Cpu, RefreshCw, FolderOpen, AlertCircle, CheckCircle2 } from "lucide-react";

interface Conversation {
  id: string;
  title: string;
  created_at: number;
  updated_at: number;
  model?: string;
}

interface ChatMessage {
  id: number;
  conversation_id: string;
  role: string;
  content: string;
  timestamp: number;
  model?: string;
  tokens?: number;
}

interface PromptTemplate {
  id: number;
  name: string;
  template: string;
  description?: string;
  created_at: number;
}

interface OllamaModel {
  name: string;
  size: number;
  modified_at: string;
  digest: string;
}

export default function AIConsciousness() {
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [selectedConversation, setSelectedConversation] = useState<string | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [inputMessage, setInputMessage] = useState("");
  const [templates, setTemplates] = useState<PromptTemplate[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedModel, setSelectedModel] = useState<string | null>(null);
  const [ollamaModels, setOllamaModels] = useState<OllamaModel[]>([]);
  const [ollamaStatus, setOllamaStatus] = useState<boolean | null>(null);
  const [modelsFolder, setModelsFolder] = useState<string>("");
  const [availableModelFiles, setAvailableModelFiles] = useState<string[]>([]);
  const [isSending, setIsSending] = useState(false);
  
  // Modal states
  const [showConversationModal, setShowConversationModal] = useState(false);
  const [showTemplateModal, setShowTemplateModal] = useState(false);
  const [conversationTitle, setConversationTitle] = useState("");
  const [templateName, setTemplateName] = useState("");
  const [templateContent, setTemplateContent] = useState("");
  const [templateDescription, setTemplateDescription] = useState("");

  useEffect(() => {
    loadConversations();
    loadTemplates();
    checkOllama();
    loadModels();
    getModelsFolder();
    scanModelsFolder();
  }, []);

  useEffect(() => {
    if (selectedConversation) {
      loadMessages(selectedConversation);
    }
  }, [selectedConversation]);

  const checkOllama = async () => {
    try {
      const status = await invoke<boolean>("check_ollama_status");
      setOllamaStatus(status);
      if (status) {
        await loadModels();
      }
    } catch (error) {
      console.error("Failed to check Ollama status:", error);
      setOllamaStatus(false);
    }
  };

  const loadModels = async () => {
    try {
      const models = await invoke<OllamaModel[]>("list_ollama_models");
      setOllamaModels(models);
      if (models.length > 0 && !selectedModel) {
        setSelectedModel(models[0].name);
      }
    } catch (error) {
      console.error("Failed to load Ollama models:", error);
    }
  };

  const getModelsFolder = async () => {
    try {
      const folder = await invoke<string>("get_models_folder_path");
      setModelsFolder(folder);
    } catch (error) {
      console.error("Failed to get models folder:", error);
    }
  };

  const scanModelsFolder = async () => {
    try {
      const files = await invoke<string[]>("scan_models_folder");
      setAvailableModelFiles(files);
    } catch (error) {
      console.error("Failed to scan models folder:", error);
    }
  };

  const loadConversations = async () => {
    try {
      const data = await invoke<Conversation[]>("list_conversations");
      setConversations(data);
      if (data.length > 0 && !selectedConversation) {
        setSelectedConversation(data[0].id);
      }
      setLoading(false);
    } catch (error) {
      console.error("Failed to load conversations:", error);
      setLoading(false);
    }
  };

  const loadMessages = async (conversationId: string) => {
    try {
      const data = await invoke<ChatMessage[]>("get_chat_messages", {
        conversationId,
      });
      setMessages(data);
    } catch (error) {
      console.error("Failed to load messages:", error);
    }
  };

  const loadTemplates = async () => {
    try {
      const data = await invoke<PromptTemplate[]>("list_prompt_templates");
      setTemplates(data);
    } catch (error) {
      console.error("Failed to load templates:", error);
    }
  };

  const handleCreateConversation = async () => {
    if (!conversationTitle.trim()) {
      alert("Please enter a conversation title");
      return;
    }

    if (conversationTitle.trim().length > 100) {
      alert("Title is too long (max 100 characters)");
      return;
    }

    try {
      const id = `conv_${Date.now()}`;
      await invoke("create_conversation", {
        id,
        title: conversationTitle.trim(),
        model: selectedModel,
      });
      await loadConversations();
      setSelectedConversation(id);
      setConversationTitle("");
      setShowConversationModal(false);
    } catch (error) {
      console.error("Failed to create conversation:", error);
      alert(`Failed to create conversation: ${error}`);
    }
  };

  const handleSendMessage = async () => {
    if (!inputMessage.trim() || !selectedConversation) return;
    if (!selectedModel) {
      alert("Please select a model first");
      return;
    }
    if (!ollamaStatus) {
      alert("Ollama is not running. Please start Ollama first.");
      return;
    }

    if (inputMessage.length > 10000) {
      alert("Message is too long (max 10000 characters)");
      return;
    }

    setIsSending(true);
    const userMessage = inputMessage;
    setInputMessage("");

    try {
      // Add user message
      await invoke("add_chat_message", {
        conversationId: selectedConversation,
        role: "user",
        content: userMessage,
        model: selectedModel,
        tokens: null,
      });

      // Get conversation history for context
      const history = await invoke<ChatMessage[]>("get_chat_messages", {
        conversationId: selectedConversation,
      });

      // Prepare messages for Ollama (last 10 messages for context)
      const recentMessages = history.slice(-10).map(msg => ({
        role: msg.role,
        content: msg.content,
      }));

      // Call Ollama
      const aiResponse = await invoke<string>("chat_with_ollama", {
        model: selectedModel,
        messages: recentMessages,
      });

      // Add AI response
      await invoke("add_chat_message", {
        conversationId: selectedConversation,
        role: "assistant",
        content: aiResponse,
        model: selectedModel,
        tokens: aiResponse.split(" ").length,
      });

      await loadMessages(selectedConversation);
    } catch (error) {
      console.error("Failed to send message:", error);
      alert(`Failed to send message: ${error}`);
      setInputMessage(userMessage); // Restore message on error
    } finally {
      setIsSending(false);
    }
  };

  const handleLoadModel = async (filePath: string) => {
    try {
      const result = await invoke<string>("load_model_from_file", {
        filePath,
      });
      alert(result);
      await scanModelsFolder();
      await loadModels();
    } catch (error) {
      console.error("Failed to load model:", error);
      alert(`Failed to load model: ${error}`);
    }
  };

  const handleCreateTemplate = async () => {
    if (!templateName.trim()) {
      alert("Please enter a template name");
      return;
    }

    if (!/^[a-zA-Z0-9_\s-]+$/.test(templateName.trim())) {
      alert("Template name can only contain letters, numbers, underscores, hyphens, and spaces");
      return;
    }

    if (!templateContent.trim()) {
      alert("Please enter template content");
      return;
    }

    if (templateContent.length > 50000) {
      alert("Template content is too long (max 50000 characters)");
      return;
    }

    if (templateDescription && templateDescription.length > 500) {
      alert("Description is too long (max 500 characters)");
      return;
    }

    try {
      await invoke("create_prompt_template", {
        name: templateName.trim(),
        template: templateContent.trim(),
        description: templateDescription.trim() || undefined,
      });
      await loadTemplates();
      setTemplateName("");
      setTemplateContent("");
      setTemplateDescription("");
      setShowTemplateModal(false);
    } catch (error) {
      console.error("Failed to create template:", error);
      alert(`Failed to create template: ${error}`);
    }
  };

  const formatTime = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleTimeString();
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  if (loading) {
    return <div className="text-center">Loading...</div>;
  }

  return (
    <div className="flex h-[calc(100vh-8rem)] gap-6">
      {/* Sidebar */}
      <div className="w-80 space-y-4 overflow-y-auto">
        {/* Ollama Status */}
        <Card title="Ollama Status">
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-400">Status:</span>
              <div className="flex items-center gap-2">
                {ollamaStatus ? (
                  <>
                    <CheckCircle2 className="w-4 h-4 text-neon-green" />
                    <span className="text-sm text-neon-green">Running</span>
                  </>
                ) : (
                  <>
                    <AlertCircle className="w-4 h-4 text-neon-red" />
                    <span className="text-sm text-neon-red">Not Running</span>
                  </>
                )}
              </div>
            </div>
            <Button
              variant="secondary"
              className="w-full"
              onClick={checkOllama}
            >
              <RefreshCw className="w-4 h-4 mr-2" />
              Check Status
            </Button>
          </div>
        </Card>

        {/* Model Selection */}
        <Card title="Models">
          <div className="space-y-3">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm text-gray-400">Available Models:</span>
              <Button
                variant="secondary"
                onClick={loadModels}
                className="p-1"
              >
                <RefreshCw className="w-3 h-3" />
              </Button>
            </div>
            <select
              value={selectedModel || ""}
              onChange={(e) => setSelectedModel(e.target.value || null)}
              className="glass-input w-full text-sm"
              disabled={!ollamaStatus || ollamaModels.length === 0}
            >
              <option value="">Select a model...</option>
              {ollamaModels.map((model) => (
                <option key={model.name} value={model.name}>
                  {model.name} ({formatBytes(model.size)})
                </option>
              ))}
            </select>
            {ollamaModels.length === 0 && ollamaStatus && (
              <p className="text-xs text-gray-500">
                No models loaded. Add models to Ollama first.
              </p>
            )}
          </div>
        </Card>

        {/* Models Folder */}
        <Card title="Models Folder">
          <div className="space-y-2">
            <div className="text-xs text-gray-400 break-all">
              {modelsFolder || "Loading..."}
            </div>
            <Button
              variant="secondary"
              className="w-full"
              onClick={scanModelsFolder}
            >
              <FolderOpen className="w-4 h-4 mr-2" />
              Scan Folder
            </Button>
            {availableModelFiles.length > 0 && (
              <div className="space-y-1 max-h-32 overflow-y-auto">
                <div className="text-xs text-gray-400 mb-1">Available Files:</div>
                {availableModelFiles.map((file, idx) => (
                  <div
                    key={idx}
                    className="text-xs p-2 glass-card rounded cursor-pointer hover:border-neon-cyan/50"
                    onClick={() => handleLoadModel(file)}
                    title={file}
                  >
                    <div className="truncate">{file.split("/").pop()}</div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </Card>

        {/* Conversations */}
        <Card title="Conversations">
          <div className="space-y-2">
            <Button
              variant="primary"
              className="w-full"
              onClick={() => setShowConversationModal(true)}
            >
              <Plus className="w-4 h-4 mr-2" />
              New Conversation
            </Button>
            <div className="space-y-1 max-h-96 overflow-y-auto">
              {conversations.map((conv) => (
                <button
                  key={conv.id}
                  onClick={() => setSelectedConversation(conv.id)}
                  className={`w-full text-left p-2 rounded glass-card transition-all ${
                    selectedConversation === conv.id
                      ? "border-2 border-neon-cyan"
                      : "hover:border border-white/10"
                  }`}
                >
                  <div className="font-semibold text-sm truncate">{conv.title}</div>
                  <div className="text-xs text-gray-400">
                    {new Date(conv.updated_at * 1000).toLocaleDateString()}
                  </div>
                  {conv.model && (
                    <div className="text-xs text-neon-cyan mt-1">{conv.model}</div>
                  )}
                </button>
              ))}
            </div>
          </div>
        </Card>

        {/* Prompt Templates */}
        <Card title="Prompt Templates">
          <div className="space-y-2">
            <Button
              variant="secondary"
              className="w-full"
              onClick={() => setShowTemplateModal(true)}
            >
              <Plus className="w-4 h-4 mr-2" />
              New Template
            </Button>
            <div className="space-y-1 max-h-64 overflow-y-auto">
              {templates.map((template) => (
                <div
                  key={template.id}
                  className="p-2 rounded glass-card text-sm"
                >
                  <div className="font-semibold">{template.name}</div>
                  {template.description && (
                    <div className="text-xs text-gray-400">{template.description}</div>
                  )}
                </div>
              ))}
            </div>
          </div>
        </Card>
      </div>

      {/* Main Chat Area */}
      <div className="flex-1 flex flex-col">
        {selectedConversation ? (
          <>
            <Card title="Chat" className="flex-1 flex flex-col">
              {selectedModel && (
                <div className="mb-4 p-2 glass-card rounded flex items-center gap-2">
                  <Cpu className="w-4 h-4 text-neon-cyan" />
                  <span className="text-sm text-gray-300">Model: <span className="text-neon-cyan">{selectedModel}</span></span>
                </div>
              )}
              <div className="flex-1 overflow-y-auto space-y-4 mb-4">
                {messages.length === 0 ? (
                  <div className="text-center text-gray-400 py-8">
                    <MessageSquare className="w-12 h-12 mx-auto mb-4 text-gray-500" />
                    <p>Start a conversation by sending a message</p>
                    {!selectedModel && (
                      <p className="text-xs mt-2 text-neon-amber">Please select a model first</p>
                    )}
                  </div>
                ) : (
                  messages.map((msg) => (
                    <div
                      key={msg.id}
                      className={`flex gap-3 ${
                        msg.role === "user" ? "justify-end" : "justify-start"
                      }`}
                    >
                      <div
                        className={`max-w-[80%] glass-card p-3 ${
                          msg.role === "user"
                            ? "bg-neon-cyan/20 border-neon-cyan"
                            : "bg-neon-green/20 border-neon-green"
                        }`}
                      >
                        <div className="flex items-center gap-2 mb-1">
                          {msg.role === "user" ? (
                            <User className="w-4 h-4 text-neon-cyan" />
                          ) : (
                            <Bot className="w-4 h-4 text-neon-green" />
                          )}
                          <span className="text-xs font-semibold">
                            {msg.role === "user" ? "You" : "Assistant"}
                          </span>
                          <span className="text-xs text-gray-400">
                            {formatTime(msg.timestamp)}
                          </span>
                          {msg.model && (
                            <span className="text-xs text-gray-500">({msg.model})</span>
                          )}
                        </div>
                        <p className="text-sm whitespace-pre-wrap">{msg.content}</p>
                        {msg.tokens && (
                          <div className="text-xs text-gray-500 mt-1">
                            {msg.tokens} tokens
                          </div>
                        )}
                      </div>
                    </div>
                  ))
                )}
              </div>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={inputMessage}
                  onChange={(e) => setInputMessage(e.target.value)}
                  onKeyPress={(e) => e.key === "Enter" && !isSending && handleSendMessage()}
                  className="glass-input flex-1"
                  placeholder={selectedModel ? "Type your message..." : "Select a model first..."}
                  disabled={!selectedModel || !ollamaStatus || isSending}
                />
                <Button 
                  onClick={handleSendMessage} 
                  variant="primary"
                  disabled={!selectedModel || !ollamaStatus || isSending}
                >
                  {isSending ? (
                    <RefreshCw className="w-4 h-4 animate-spin" />
                  ) : (
                    <Send className="w-4 h-4" />
                  )}
                </Button>
              </div>
            </Card>
          </>
        ) : (
          <Card title="No Conversation Selected">
            <div className="text-center py-8 text-gray-400">
              <MessageSquare className="w-12 h-12 mx-auto mb-4 text-gray-500" />
              <p>Create a new conversation to get started</p>
            </div>
          </Card>
        )}
      </div>

      {/* Create Conversation Modal */}
      <Modal
        isOpen={showConversationModal}
        onClose={() => {
          setShowConversationModal(false);
          setConversationTitle("");
        }}
        title="Create New Conversation"
      >
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Conversation Title
            </label>
            <input
              type="text"
              value={conversationTitle}
              onChange={(e) => setConversationTitle(e.target.value)}
              onKeyPress={(e) => {
                if (e.key === "Enter") {
                  handleCreateConversation();
                }
              }}
              className="glass-input w-full"
              placeholder="Enter conversation title..."
              autoFocus
            />
            <p className="text-xs text-gray-500 mt-1">
              {conversationTitle.length}/100 characters
            </p>
          </div>
          {selectedModel && (
            <div className="p-2 glass-card rounded">
              <p className="text-xs text-gray-400 mb-1">Selected Model:</p>
              <p className="text-sm text-neon-cyan">{selectedModel}</p>
            </div>
          )}
          <div className="flex gap-2 justify-end">
            <Button
              variant="secondary"
              onClick={() => {
                setShowConversationModal(false);
                setConversationTitle("");
              }}
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              onClick={handleCreateConversation}
              disabled={!conversationTitle.trim()}
            >
              Create
            </Button>
          </div>
        </div>
      </Modal>

      {/* Create Template Modal */}
      <Modal
        isOpen={showTemplateModal}
        onClose={() => {
          setShowTemplateModal(false);
          setTemplateName("");
          setTemplateContent("");
          setTemplateDescription("");
        }}
        title="Create Prompt Template"
        className="max-w-2xl"
      >
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Template Name
            </label>
            <input
              type="text"
              value={templateName}
              onChange={(e) => setTemplateName(e.target.value)}
              className="glass-input w-full"
              placeholder="e.g., Code Review, Email Draft..."
            />
            <p className="text-xs text-gray-500 mt-1">
              Letters, numbers, underscores, hyphens, and spaces only
            </p>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Description (Optional)
            </label>
            <input
              type="text"
              value={templateDescription}
              onChange={(e) => setTemplateDescription(e.target.value)}
              className="glass-input w-full"
              placeholder="Brief description of the template..."
            />
            <p className="text-xs text-gray-500 mt-1">
              {templateDescription.length}/500 characters
            </p>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Template Content
            </label>
            <textarea
              value={templateContent}
              onChange={(e) => setTemplateContent(e.target.value)}
              className="glass-input w-full min-h-[200px] font-mono text-sm"
              placeholder="Enter your template content here..."
            />
            <p className="text-xs text-gray-500 mt-1">
              {templateContent.length}/50000 characters
            </p>
          </div>
          <div className="flex gap-2 justify-end">
            <Button
              variant="secondary"
              onClick={() => {
                setShowTemplateModal(false);
                setTemplateName("");
                setTemplateContent("");
                setTemplateDescription("");
              }}
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              onClick={handleCreateTemplate}
              disabled={!templateName.trim() || !templateContent.trim()}
            >
              Create Template
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
