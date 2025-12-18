import { Link } from "react-router-dom";
import Card from "../ui/Card";
import {
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

const modules = [
  { path: "/system-monitor", icon: Activity, label: "System Monitor", color: "cyan" },
  { path: "/network", icon: Network, label: "Network", color: "green" },
  { path: "/ai", icon: Brain, label: "AI Consciousness", color: "amber" },
  { path: "/devops", icon: Wrench, label: "DevOps", color: "cyan" },
  { path: "/automation", icon: Zap, label: "Automation", color: "green" },
  { path: "/packages", icon: Package, label: "Packages", color: "amber" },
  { path: "/reality", icon: Clock, label: "Reality & Timeline", color: "cyan" },
  { path: "/vector-store", icon: Database, label: "Vector Store", color: "green" },
  { path: "/security", icon: Shield, label: "Security", color: "red" },
  { path: "/utilities", icon: Settings, label: "Utilities", color: "amber" },
  { path: "/create", icon: Palette, label: "Create Hub", color: "cyan" },
  { path: "/testing", icon: TestTube, label: "Testing", color: "green" },
  { path: "/config", icon: Cog, label: "Config", color: "amber" },
  { path: "/migration", icon: GitBranch, label: "Migration", color: "cyan" },
  { path: "/websocket", icon: Radio, label: "WebSocket", color: "green" },
  { path: "/errors", icon: AlertTriangle, label: "Errors", color: "red" },
  { path: "/rate-limit", icon: Timer, label: "Rate Limit", color: "amber" },
  { path: "/vector-search", icon: Search, label: "Vector Search", color: "cyan" },
  { path: "/analytics", icon: BarChart3, label: "Analytics", color: "green" },
];

export default function RadialHub() {
  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-4xl font-bold mb-2 phosphor-glow-cyan">
          MINA Dashboard
        </h1>
        <p className="text-gray-400">Comprehensive System Assistant & Monitoring Platform</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
        {modules.map((module) => {
          const Icon = module.icon;
          const colorClass = `text-neon-${module.color}`;
          return (
            <Link key={module.path} to={module.path}>
              <Card className="hover:scale-105 transition-transform cursor-pointer">
                <div className="flex flex-col items-center text-center">
                  <Icon className={`w-12 h-12 mb-4 ${colorClass}`} />
                  <h3 className="text-lg font-semibold">{module.label}</h3>
                </div>
              </Card>
            </Link>
          );
        })}
      </div>
    </div>
  );
}

