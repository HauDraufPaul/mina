import { Link, useLocation } from "react-router-dom";
import {
  Home,
  Activity,
  Network,
  Brain,
  Wrench,
  Zap,
  Package,
  Clock,
  Database,
  Shield,
  Settings,
  Palette,
  TestTube,
  Cog,
  GitBranch,
  Radio,
  AlertTriangle,
  Timer,
  Search,
  BarChart3,
} from "lucide-react";

const navItems = [
  { path: "/", icon: Home, label: "Dashboard" },
  { path: "/system-monitor", icon: Activity, label: "System Monitor" },
  { path: "/network", icon: Network, label: "Network" },
  { path: "/ai", icon: Brain, label: "AI Consciousness" },
  { path: "/devops", icon: Wrench, label: "DevOps" },
  { path: "/automation", icon: Zap, label: "Automation" },
  { path: "/packages", icon: Package, label: "Packages" },
  { path: "/reality", icon: Clock, label: "Reality & Timeline" },
  { path: "/vector-store", icon: Database, label: "Vector Store" },
  { path: "/security", icon: Shield, label: "Security" },
  { path: "/utilities", icon: Settings, label: "Utilities" },
  { path: "/create", icon: Palette, label: "Create Hub" },
  { path: "/testing", icon: TestTube, label: "Testing" },
  { path: "/config", icon: Cog, label: "Config" },
  { path: "/migration", icon: GitBranch, label: "Migration" },
  { path: "/websocket", icon: Radio, label: "WebSocket" },
  { path: "/errors", icon: AlertTriangle, label: "Errors" },
  { path: "/rate-limit", icon: Timer, label: "Rate Limit" },
  { path: "/vector-search", icon: Search, label: "Vector Search" },
  { path: "/analytics", icon: BarChart3, label: "Analytics" },
];

export default function Sidebar() {
  const location = useLocation();

  return (
    <aside className="w-64 glass-card p-4 mr-6">
      <nav className="space-y-2">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = location.pathname === item.path;
          return (
            <Link
              key={item.path}
              to={item.path}
              className={`flex items-center gap-3 px-4 py-2 rounded-md transition-all ${
                isActive
                  ? "bg-neon-cyan/20 border border-neon-cyan text-neon-cyan"
                  : "text-gray-300 hover:bg-white/5 hover:text-white"
              }`}
            >
              <Icon className="w-5 h-5" />
              <span className="text-sm">{item.label}</span>
            </Link>
          );
        })}
      </nav>
    </aside>
  );
}

