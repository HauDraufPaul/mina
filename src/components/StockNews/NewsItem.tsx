import { useState } from "react";
import { StockNewsItem } from "../../stores/stockNewsStore";
import { ExternalLink, ChevronDown, ChevronUp } from "lucide-react";

interface NewsItemProps {
  item: StockNewsItem;
}

export default function NewsItem({ item }: NewsItemProps) {
  const [expanded, setExpanded] = useState(false);

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diff = Math.floor((now.getTime() - date.getTime()) / 1000);

    if (diff < 60) return `${diff}s ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return date.toLocaleDateString();
  };

  const getSentimentColor = (sentiment?: number) => {
    if (!sentiment) return "bg-gray-500/20 text-gray-400";
    if (sentiment > 0.2) return "bg-green-500/20 text-green-400";
    if (sentiment < -0.2) return "bg-red-500/20 text-red-400";
    return "bg-gray-500/20 text-gray-400";
  };

  const stripHtml = (html: string) => {
    const tmp = document.createElement("DIV");
    tmp.innerHTML = html;
    return tmp.textContent || tmp.innerText || "";
  };

  const getPreview = (content: string) => {
    const text = stripHtml(content);
    return text.length > 200 ? text.substring(0, 200) + "..." : text;
  };

  return (
    <div className="p-4 hover:bg-white/5 transition-colors cursor-pointer">
      <div onClick={() => setExpanded(!expanded)}>
        {/* Header */}
        <div className="flex items-start justify-between gap-3 mb-2">
          <div className="flex-1">
            <h3 className="text-white font-semibold mb-1 hover:text-neon-cyan transition-colors">
              {item.title}
            </h3>
            
            {/* Tickers */}
            <div className="flex items-center gap-2 flex-wrap mb-2">
              {item.tickers.map((ticker) => (
                <span
                  key={ticker}
                  className="text-xs font-bold text-neon-cyan bg-neon-cyan/10 
                           px-2 py-1 rounded border border-neon-cyan/20"
                >
                  {ticker}
                </span>
              ))}
            </div>

            {/* Metadata */}
            <div className="flex items-center gap-3 text-xs text-gray-400 font-mono">
              <span className="text-neon-cyan">{item.source}</span>
              <span>•</span>
              <span>{formatTime(item.published_at)}</span>
              {item.sentiment !== undefined && item.sentiment !== null && (
                <>
                  <span>•</span>
                  <span className={`px-2 py-0.5 rounded ${getSentimentColor(item.sentiment)}`}>
                    Sentiment: {item.sentiment > 0 ? "+" : ""}
                    {(item.sentiment * 100).toFixed(0)}%
                  </span>
                </>
              )}
            </div>
          </div>

          <div className="flex items-center gap-2">
            <a
              href={item.url}
              target="_blank"
              rel="noopener noreferrer"
              onClick={(e) => e.stopPropagation()}
              className="text-gray-400 hover:text-neon-cyan transition-colors"
            >
              <ExternalLink className="w-4 h-4" />
            </a>
            {expanded ? (
              <ChevronUp className="w-4 h-4 text-gray-400" />
            ) : (
              <ChevronDown className="w-4 h-4 text-gray-400" />
            )}
          </div>
        </div>

        {/* Preview/Content */}
        {!expanded && (
          <p className="text-sm text-gray-400 mt-2">
            {getPreview(item.content)}
          </p>
        )}
      </div>

      {/* Expanded Content */}
      {expanded && (
        <div className="mt-3 pt-3 border-t border-white/10">
          <div
            className="prose prose-invert prose-sm max-w-none
                     prose-p:text-gray-300 prose-a:text-neon-cyan 
                     prose-headings:text-white"
            dangerouslySetInnerHTML={{ __html: item.content }}
          />
          
          <div className="mt-4 flex items-center gap-2">
            <a
              href={item.url}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 text-sm text-neon-cyan 
                       hover:text-neon-cyan/80 transition-colors"
            >
              Read full article
              <ExternalLink className="w-3 h-3" />
            </a>
          </div>
        </div>
      )}
    </div>
  );
}

