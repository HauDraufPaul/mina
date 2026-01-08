import { ReactNode } from "react";
import Navbar from "./Navbar";
import Sidebar from "./Sidebar";
import CommandBar from "../CommandBar/CommandBar";
import { TickerTape } from "../StockNews";

interface LayoutProps {
  children: ReactNode;
}

export default function Layout({ children }: LayoutProps) {
  return (
    <div className="min-h-screen bg-[#0a0a0a] text-white font-mono">
      <Navbar />
      <CommandBar />
      <div className="flex">
        <Sidebar />
        <main className="flex-1 p-6 overflow-y-auto pb-20">
          {children}
        </main>
      </div>
      <TickerTape />
    </div>
  );
}

