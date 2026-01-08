import { useState, useEffect } from "react";
import { Link, useLocation } from "react-router-dom";
import type { LucideIcon } from "lucide-react";
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
  ChevronRight,
  ChevronDown,
  Layers,
  Gauge,
  Newspaper,
} from "lucide-react";

interface MenuItem {
  path: string;
  icon: LucideIcon;
  label: string;
}

interface MenuCategory {
  id: string;
  label: string;
  icon: LucideIcon;
  items: MenuItem[];
  defaultExpanded?: boolean;
}

type MenuEntry = MenuItem | MenuCategory;

const isCategory = (entry: MenuEntry): entry is MenuCategory => {
  return "items" in entry;
};

const menuStructure: MenuEntry[] = [
  { path: "/", icon: Home, label: "Dashboard" },
  {
    id: "foundation",
    label: "FOUNDATION",
    icon: Layers,
    items: [
      { path: "/system-monitor", icon: Activity, label: "System Monitor" },
      { path: "/network", icon: Network, label: "Network" },
      { path: "/ai", icon: Brain, label: "AI Consciousness" },
      { path: "/devops", icon: Wrench, label: "DevOps" },
      { path: "/automation", icon: Zap, label: "Automation" },
      { path: "/packages", icon: Package, label: "Packages" },
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
    ],
  },
  {
    id: "temporal-engine",
    label: "TEMPORAL ENGINE",
    icon: Gauge,
    items: [
      { path: "/reality", icon: Clock, label: "Reality & Timeline" },
    ],
  },
  {
    id: "market-intelligence",
    label: "MARKET INTELLIGENCE",
    icon: Gauge,
    items: [
      { path: "/stock-news", icon: Newspaper, label: "Stock News" },
    ],
  },
];

export default function Sidebar() {
  const location = useLocation();
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(
    new Set()
  );

  // Auto-expand categories if current route matches a child item
  useEffect(() => {
    const newExpanded = new Set<string>();
    menuStructure.forEach((entry) => {
      if (isCategory(entry)) {
        const hasActiveChild = entry.items.some(
          (item) => item.path === location.pathname
        );
        if (hasActiveChild) {
          newExpanded.add(entry.id);
        }
      }
    });
    setExpandedCategories(newExpanded);
  }, [location.pathname]);

  const toggleCategory = (categoryId: string) => {
    setExpandedCategories((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(categoryId)) {
        newSet.delete(categoryId);
      } else {
        newSet.add(categoryId);
      }
      return newSet;
    });
  };

  const isCategoryExpanded = (categoryId: string) => {
    return expandedCategories.has(categoryId);
  };

  const isItemActive = (path: string) => {
    return location.pathname === path;
  };

  const isCategoryActive = (category: MenuCategory) => {
    return category.items.some((item) => isItemActive(item.path));
  };

  const renderMenuItem = (item: MenuItem, indent: boolean = false) => {
    const Icon = item.icon;
    const isActive = isItemActive(item.path);
    return (
      <Link
        key={item.path}
        to={item.path}
        className={`flex items-center gap-3 px-4 py-2 rounded-md transition-all ${
          indent ? "ml-6" : ""
        } ${
          isActive
            ? "bg-neon-cyan/20 border border-neon-cyan text-neon-cyan"
            : "text-gray-300 hover:bg-white/5 hover:text-white"
        }`}
      >
        <Icon className="w-5 h-5" />
        <span className="text-sm">{item.label}</span>
      </Link>
    );
  };

  const renderCategory = (category: MenuCategory) => {
    const isExpanded = isCategoryExpanded(category.id);
    const isActive = isCategoryActive(category);
    const CategoryIcon = category.icon;
    const ChevronIcon = isExpanded ? ChevronDown : ChevronRight;

    return (
      <div key={category.id} className="space-y-1">
        <button
          onClick={() => toggleCategory(category.id)}
          className={`w-full flex items-center gap-3 px-4 py-2 rounded-md transition-all ${
            isActive
              ? "bg-neon-cyan/10 border border-neon-cyan/30 text-neon-cyan"
              : "text-gray-300 hover:bg-white/5 hover:text-white"
          }`}
          aria-expanded={isExpanded}
          aria-label={`Toggle ${category.label} category`}
        >
          <ChevronIcon className="w-4 h-4 transition-transform" />
          <CategoryIcon className="w-5 h-5" />
          <span className="text-sm font-semibold">{category.label}</span>
        </button>
        {isExpanded && (
          <div className="space-y-1 transition-all duration-200 ease-in-out">
            {category.items.map((item) => renderMenuItem(item, true))}
          </div>
        )}
      </div>
    );
  };

  return (
    <aside className="w-64 glass-card p-4 mr-6 sticky top-[88px] self-start h-[calc(100vh-88px)] overflow-y-auto">
      <nav className="space-y-2">
        {menuStructure.map((entry) => {
          if (isCategory(entry)) {
            return renderCategory(entry);
          } else {
            return renderMenuItem(entry);
          }
        })}
      </nav>
    </aside>
  );
}

