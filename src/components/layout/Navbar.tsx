import { Link } from "react-router-dom";
import { Activity, Settings } from "lucide-react";

export default function Navbar() {
  return (
    <nav className="glass-card p-4 mb-6 sticky top-0 z-50 backdrop-blur-md bg-black/80 border-b border-white/10">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Link to="/" className="text-2xl font-bold phosphor-glow-cyan">
            MINA
          </Link>
          <span className="text-sm text-gray-400">Monitoring, Intelligence, Networking, Automation</span>
        </div>
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <Activity className="w-4 h-4 text-neon-green" />
            <span className="text-sm">System Active</span>
          </div>
          <Link to="/config" className="glass-button">
            <Settings className="w-4 h-4" />
          </Link>
        </div>
      </div>
    </nav>
  );
}

