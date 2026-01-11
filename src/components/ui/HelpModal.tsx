import Modal from "./Modal";
import Button from "./Button";
import { ExternalLink, Book } from "lucide-react";
import { HelpContent } from "./HelpButton";

interface HelpModalProps {
  isOpen: boolean;
  onClose: () => void;
  content: HelpContent;
}

export default function HelpModal({ isOpen, onClose, content }: HelpModalProps) {
  return (
    <Modal isOpen={isOpen} onClose={onClose} title={content.title}>
      <div className="space-y-6">
        {content.sections.map((section, index) => (
          <div key={index} className="space-y-2">
            <h3 className="text-lg font-semibold text-neon-cyan">{section.heading}</h3>
            <div className="text-gray-300 prose prose-invert max-w-none">
              {typeof section.content === "string" ? (
                <p className="whitespace-pre-line">{section.content}</p>
              ) : (
                section.content
              )}
            </div>
          </div>
        ))}
        
        {content.linkToDocs && (
          <div className="pt-4 border-t border-white/10">
            <a
              href={content.linkToDocs}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 text-neon-cyan hover:text-neon-cyan/80 transition-colors"
            >
              <Book className="w-4 h-4" />
              <span>View full documentation</span>
              <ExternalLink className="w-4 h-4" />
            </a>
          </div>
        )}
      </div>
    </Modal>
  );
}
