import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { Shield, Lock, Key, AlertTriangle, CheckCircle, Clock } from "lucide-react";

interface AuthAttempt {
  id: number;
  user_id: string;
  success: boolean;
  ip_address?: string;
  created_at: number;
}

export default function SecurityCenter() {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [pin, setPin] = useState("");
  const [confirmPin, setConfirmPin] = useState("");
  const [loginPin, setLoginPin] = useState("");
  const [authAttempts, setAuthAttempts] = useState<AuthAttempt[]>([]);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    checkSession();
    loadAuthAttempts();
  }, []);

  const checkSession = async () => {
    const stored = localStorage.getItem("mina_session_id");
    if (stored) {
      try {
        const valid = await invoke<boolean>("validate_session", { sessionId: stored });
        if (valid) {
          setIsAuthenticated(true);
          setSessionId(stored);
        } else {
          localStorage.removeItem("mina_session_id");
        }
      } catch (error) {
        console.error("Session validation failed:", error);
      }
    }
  };

  const loadAuthAttempts = async () => {
    try {
      const attempts = await invoke<AuthAttempt[]>("get_auth_attempts", { limit: 20 });
      setAuthAttempts(attempts);
    } catch (error) {
      console.error("Failed to load auth attempts:", error);
    }
  };

  const handleSetPin = async () => {
    if (pin.length < 4) {
      alert("PIN must be at least 4 digits");
      return;
    }
    if (pin !== confirmPin) {
      alert("PINs do not match");
      return;
    }

    setLoading(true);
    try {
      await invoke("set_pin", { userId: "default", pin });
      alert("PIN set successfully!");
      setPin("");
      setConfirmPin("");
    } catch (error) {
      alert(`Failed to set PIN: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const handleLogin = async () => {
    if (loginPin.length < 4) {
      alert("PIN must be at least 4 digits");
      return;
    }

    setLoading(true);
    try {
      const valid = await invoke<boolean>("verify_pin", {
        userId: "default",
        pin: loginPin,
      });

      if (valid) {
        const session = await invoke<string>("create_session", { userId: "default" });
        setSessionId(session);
        localStorage.setItem("mina_session_id", session);
        setIsAuthenticated(true);
        setLoginPin("");
        await loadAuthAttempts();
      } else {
        alert("Invalid PIN");
        await loadAuthAttempts();
      }
    } catch (error) {
      alert(`Login failed: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const handleLogout = () => {
    setIsAuthenticated(false);
    setSessionId(null);
    localStorage.removeItem("mina_session_id");
  };

  const formatTimestamp = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  if (!isAuthenticated) {
    return (
      <div className="space-y-6 max-w-md mx-auto">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
            Security Center
          </h1>
          <p className="text-gray-400">PIN-based authentication required</p>
        </div>

        <Card title="Login">
          <div className="space-y-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">Enter PIN</label>
              <input
                type="password"
                value={loginPin}
                onChange={(e) => setLoginPin(e.target.value)}
                className="glass-input w-full"
                placeholder="Enter your PIN"
                maxLength={10}
              />
            </div>
            <Button
              onClick={handleLogin}
              variant="primary"
              className="w-full"
              disabled={loading}
            >
              <Lock className="w-4 h-4 mr-2" />
              Login
            </Button>
          </div>
        </Card>

        <Card title="Set PIN" subtitle="First time setup">
          <div className="space-y-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">New PIN</label>
              <input
                type="password"
                value={pin}
                onChange={(e) => setPin(e.target.value)}
                className="glass-input w-full"
                placeholder="Enter new PIN (min 4 digits)"
                maxLength={10}
              />
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-2">Confirm PIN</label>
              <input
                type="password"
                value={confirmPin}
                onChange={(e) => setConfirmPin(e.target.value)}
                className="glass-input w-full"
                placeholder="Confirm PIN"
                maxLength={10}
              />
            </div>
            <Button
              onClick={handleSetPin}
              variant="secondary"
              className="w-full"
              disabled={loading}
            >
              <Key className="w-4 h-4 mr-2" />
              Set PIN
            </Button>
          </div>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
            Security Center
          </h1>
          <p className="text-gray-400">Authentication and access control</p>
        </div>
        <Button onClick={handleLogout} variant="secondary">
          Logout
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
        <Card title="Session Status" subtitle="Current session">
          <div className="flex items-center gap-2">
            <CheckCircle className="w-5 h-5 text-neon-green" />
            <span className="text-sm">Authenticated</span>
          </div>
          {sessionId && (
            <div className="mt-2 text-xs font-mono text-gray-400 truncate">
              {sessionId}
            </div>
          )}
        </Card>
        <Card title="Total Attempts" subtitle="Authentication attempts">
          <div className="text-3xl font-bold text-neon-cyan">{authAttempts.length}</div>
        </Card>
        <Card title="Failed Attempts" subtitle="Unsuccessful logins">
          <div className="text-3xl font-bold text-neon-red">
            {authAttempts.filter((a) => !a.success).length}
          </div>
        </Card>
      </div>

      <Card title="Recent Authentication Attempts">
        <div className="space-y-3">
          {authAttempts.length === 0 ? (
            <div className="text-center text-gray-400 py-8">No attempts recorded</div>
          ) : (
            authAttempts.map((attempt) => (
              <div
                key={attempt.id}
                className="glass-card p-4 flex items-center justify-between"
              >
                <div className="flex items-center gap-3">
                  {attempt.success ? (
                    <CheckCircle className="w-5 h-5 text-neon-green" />
                  ) : (
                    <AlertTriangle className="w-5 h-5 text-neon-red" />
                  )}
                  <div>
                    <div className="font-semibold">
                      {attempt.success ? "Success" : "Failed"}
                    </div>
                    <div className="text-xs text-gray-400">
                      User: {attempt.user_id} • {formatTimestamp(attempt.created_at)}
                    </div>
                    {attempt.ip_address && (
                      <div className="text-xs text-gray-500 font-mono">
                        IP: {attempt.ip_address}
                      </div>
                    )}
                  </div>
                </div>
                <div className="text-xs text-gray-400">
                  <Clock className="w-4 h-4 inline mr-1" />
                  {formatTimestamp(attempt.created_at)}
                </div>
              </div>
            ))
          )}
        </div>
      </Card>

      <Card title="Permission Management">
        <div className="space-y-4">
          <div className="text-sm text-gray-400 mb-4">
            Manage user permissions and access controls
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div className="glass-card p-4">
              <div className="text-sm font-semibold mb-2">Resources</div>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span>System Monitor</span>
                  <span className="text-neon-green">✓</span>
                </div>
                <div className="flex justify-between">
                  <span>Network</span>
                  <span className="text-neon-green">✓</span>
                </div>
                <div className="flex justify-between">
                  <span>Configuration</span>
                  <span className="text-neon-green">✓</span>
                </div>
              </div>
            </div>
            <div className="glass-card p-4">
              <div className="text-sm font-semibold mb-2">Actions</div>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span>Read</span>
                  <span className="text-neon-green">✓</span>
                </div>
                <div className="flex justify-between">
                  <span>Write</span>
                  <span className="text-neon-green">✓</span>
                </div>
                <div className="flex justify-between">
                  <span>Delete</span>
                  <span className="text-neon-amber">⚠</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </Card>

      <Card title="Security Settings">
        <div className="space-y-4">
          <div className="flex items-center justify-between p-3 glass-card">
            <div className="flex items-center gap-2">
              <Shield className="w-5 h-5 text-neon-cyan" />
              <span>Session Timeout</span>
            </div>
            <span className="font-mono text-sm">1 hour</span>
          </div>
          <div className="flex items-center justify-between p-3 glass-card">
            <div className="flex items-center gap-2">
              <Lock className="w-5 h-5 text-neon-green" />
              <span>PIN Requirements</span>
            </div>
            <span className="text-sm">Minimum 4 digits</span>
          </div>
          <div className="flex items-center justify-between p-3 glass-card">
            <div className="flex items-center gap-2">
              <AlertTriangle className="w-5 h-5 text-neon-amber" />
              <span>Failed Attempt Limit</span>
            </div>
            <span className="font-mono text-sm">No limit</span>
          </div>
        </div>
      </Card>
    </div>
  );
}
