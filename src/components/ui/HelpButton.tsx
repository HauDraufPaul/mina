import { useState } from "react";
import { HelpCircle } from "lucide-react";
import Button from "./Button";
import HelpModal from "./HelpModal";

export interface HelpContent {
  title: string;
  sections: Array<{
    heading: string;
    content: string | React.ReactNode;
  }>;
  linkToDocs?: string; // Docusaurus path, e.g., "/docs/modules/automation-circuit"
}

interface HelpButtonProps {
  content: HelpContent;
  variant?: "icon" | "button";
  className?: string;
}

export default function HelpButton({ content, variant = "icon", className = "" }: HelpButtonProps) {
  const [isOpen, setIsOpen] = useState(false);

  if (variant === "icon") {
    return (
      <>
        <button
          onClick={() => setIsOpen(true)}
          className={`p-2 rounded-md hover:bg-white/5 text-gray-400 hover:text-neon-cyan transition-colors ${className}`}
          aria-label="Show help"
        >
          <HelpCircle className="w-5 h-5" />
        </button>
        <HelpModal isOpen={isOpen} onClose={() => setIsOpen(false)} content={content} />
      </>
    );
  }

  return (
    <>
      <Button variant="secondary" onClick={() => setIsOpen(true)} className={className}>
        <HelpCircle className="w-4 h-4 mr-2" />
        Help
      </Button>
      <HelpModal isOpen={isOpen} onClose={() => setIsOpen(false)} content={content} />
    </>
  );
}
