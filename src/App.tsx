import React from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "./components/ui/Toast";
import Layout from "./components/layout/Layout";
import { notificationService } from "./services/notificationService";
import RadialHub from "./components/RadialHub/RadialHub";
import SystemMonitorHub from "./components/modules/SystemMonitorHub/SystemMonitorHub";
import NetworkConstellation from "./components/modules/NetworkConstellation/NetworkConstellation";
import AIConsciousness from "./components/modules/AIConsciousness/AIConsciousness";
import DevOpsControl from "./components/modules/DevOpsControl/DevOpsControl";
import AutomationCircuit from "./components/modules/AutomationCircuit/AutomationCircuit";
import PackagesRepository from "./components/modules/PackagesRepository/PackagesRepository";
import RealityTimelineStudio from "./components/modules/RealityTimelineStudio/RealityTimelineStudio";
import VectorStoreManager from "./components/modules/VectorStoreManager/VectorStoreManager";
import SecurityCenter from "./components/modules/SecurityCenter/SecurityCenter";
import SystemUtilities from "./components/modules/SystemUtilities/SystemUtilities";
import CreateHub from "./components/modules/CreateHub/CreateHub";
import TestingCenter from "./components/modules/TestingCenter/TestingCenter";
import ConfigurationManager from "./components/modules/ConfigurationManager/ConfigurationManager";
import MigrationManager from "./components/modules/MigrationManager/MigrationManager";
import WebSocketMonitor from "./components/modules/WebSocketMonitor/WebSocketMonitor";
import ErrorDashboard from "./components/modules/ErrorDashboard/ErrorDashboard";
import RateLimitMonitor from "./components/modules/RateLimitMonitor/RateLimitMonitor";
import VectorSearch from "./components/modules/VectorSearch/VectorSearch";
import AdvancedAnalytics from "./components/modules/AdvancedAnalytics/AdvancedAnalytics";
import GridLayoutView from "./components/GridLayout/GridLayoutView";
import { NewsPanel } from "./components/StockNews";
import MarketDataHub from "./components/modules/MarketDataHub/MarketDataHub";
import PortfolioManager from "./components/modules/PortfolioManager/PortfolioManager";
import EconomicCalendar from "./components/modules/EconomicCalendar/EconomicCalendar";
import ChartStudio from "./components/modules/ChartStudio/ChartStudio";
import MessagingHub from "./components/modules/Messaging/MessagingHub";
import SentimentAnalysis from "./components/modules/SentimentAnalysis/SentimentAnalysis";
import DocumentationViewer from "./components/modules/Documentation/DocumentationViewer";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
    },
  },
});

function App() {
  // Setup notification service
  React.useEffect(() => {
    // Request notification permission on app start
    notificationService.requestPermission();
    
    // Setup event listeners for notifications
    const cleanup = notificationService.setupEventListeners();
    
    return () => {
      cleanup();
    };
  }, []);

  return (
    <QueryClientProvider client={queryClient}>
      <ToastProvider>
        <BrowserRouter>
          <Layout>
            <Routes>
            <Route path="/" element={<RadialHub />} />
            <Route path="/system-monitor" element={<SystemMonitorHub />} />
            <Route path="/network" element={<NetworkConstellation />} />
            <Route path="/ai" element={<AIConsciousness />} />
            <Route path="/devops" element={<DevOpsControl />} />
            <Route path="/automation" element={<AutomationCircuit />} />
            <Route path="/packages" element={<PackagesRepository />} />
            <Route path="/reality" element={<RealityTimelineStudio />} />
            <Route path="/vector-store" element={<VectorStoreManager />} />
            <Route path="/security" element={<SecurityCenter />} />
            <Route path="/utilities" element={<SystemUtilities />} />
            <Route path="/create" element={<CreateHub />} />
            <Route path="/testing" element={<TestingCenter />} />
            <Route path="/config" element={<ConfigurationManager />} />
            <Route path="/migration" element={<MigrationManager />} />
            <Route path="/websocket" element={<WebSocketMonitor />} />
            <Route path="/errors" element={<ErrorDashboard />} />
            <Route path="/rate-limit" element={<RateLimitMonitor />} />
            <Route path="/vector-search" element={<VectorSearch />} />
            <Route path="/analytics" element={<AdvancedAnalytics />} />
            <Route path="/grid" element={<GridLayoutView />} />
            <Route path="/stock-news" element={<NewsPanel />} />
            <Route path="/market-data" element={<MarketDataHub />} />
            <Route path="/portfolio" element={<PortfolioManager />} />
            <Route path="/economic-calendar" element={<EconomicCalendar />} />
            <Route path="/chart-studio" element={<ChartStudio />} />
            <Route path="/messaging" element={<MessagingHub />} />
            <Route path="/sentiment" element={<SentimentAnalysis />} />
            <Route path="/docs" element={<DocumentationViewer />} />
            <Route path="/docs/:pageId" element={<DocumentationViewer />} />
          </Routes>
        </Layout>
      </BrowserRouter>
      </ToastProvider>
    </QueryClientProvider>
  );
}

export default App;

