import { useEffect, useRef, useState } from "react";
import { useStockNewsStore } from "../../stores/stockNewsStore";

export default function TickerTape() {
  const { 
    getFilteredNews, 
    tickerTapeSpeed, 
    tickerTapePaused,
    setTickerTapePaused 
  } = useStockNewsStore();
  
  const containerRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const [scrollPosition, setScrollPosition] = useState(0);
  const animationRef = useRef<number>();

  const news = getFilteredNews().slice(0, 50); // Limit to 50 most recent items

  // Animation loop
  useEffect(() => {
    if (tickerTapePaused || !contentRef.current) return;

    const animate = () => {
      setScrollPosition((prev) => {
        const contentWidth = contentRef.current?.scrollWidth || 0;
        const containerWidth = containerRef.current?.clientWidth || 0;
        
        // Reset when fully scrolled
        if (prev <= -(contentWidth - containerWidth)) {
          return 0;
        }
        
        // Move by speed (pixels per frame at 60fps)
        return prev - tickerTapeSpeed / 60;
      });

      animationRef.current = requestAnimationFrame(animate);
    };

    animationRef.current = requestAnimationFrame(animate);

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [tickerTapeSpeed, tickerTapePaused]);

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diff = Math.floor((now.getTime() - date.getTime()) / 1000);

    if (diff < 60) return `${diff}s ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
  };

  const getSentimentColor = (sentiment?: number) => {
    if (!sentiment) return "text-white";
    if (sentiment > 0.2) return "text-neon-green";
    if (sentiment < -0.2) return "text-neon-red";
    return "text-white";
  };

  if (news.length === 0) {
    return (
      <div className="fixed bottom-0 left-0 right-0 z-50 bg-black/90 border-t border-white/10 backdrop-blur-sm">
        <div className="h-12 flex items-center justify-center">
          <p className="text-gray-500 text-sm font-mono">No news available</p>
        </div>
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
      className="fixed bottom-0 left-0 right-0 z-50 bg-black/90 border-t border-white/10 backdrop-blur-sm overflow-hidden cursor-pointer"
      onMouseEnter={() => setTickerTapePaused(true)}
      onMouseLeave={() => setTickerTapePaused(false)}
    >
      <div className="h-12 relative">
        <div
          ref={contentRef}
          className="absolute top-0 left-0 h-full flex items-center gap-8"
          style={{
            transform: `translateX(${scrollPosition}px)`,
            willChange: "transform",
          }}
        >
          {/* Duplicate content for seamless loop */}
          {[...news, ...news].map((item, index) => (
            <div
              key={`${item.id}-${index}`}
              className="flex items-center gap-3 whitespace-nowrap group"
              onClick={() => window.open(item.url, "_blank")}
            >
              {/* Tickers */}
              <div className="flex items-center gap-1">
                {item.tickers.slice(0, 2).map((ticker) => (
                  <span
                    key={ticker}
                    className={`text-xs font-bold ${getSentimentColor(item.sentiment)} 
                              bg-white/5 px-2 py-1 rounded border border-white/10
                              group-hover:bg-white/10 transition-colors`}
                  >
                    {ticker}
                  </span>
                ))}
              </div>

              {/* Headline */}
              <span className="text-sm text-gray-300 group-hover:text-white transition-colors">
                {item.title}
              </span>

              {/* Source & Time */}
              <div className="flex items-center gap-2 text-xs text-gray-500">
                <span className="text-neon-cyan">{item.source}</span>
                <span>•</span>
                <span>{formatTime(item.published_at)}</span>
              </div>

              {/* Separator */}
              <span className="text-white/20">|</span>
            </div>
          ))}
        </div>
      </div>

      {/* Pause indicator */}
      {tickerTapePaused && (
        <div className="absolute top-2 right-4">
          <div className="bg-amber-500/20 border border-amber-500/30 rounded px-2 py-1">
            <span className="text-xs text-amber-400 font-mono">PAUSED</span>
          </div>
        </div>
      )}
    </div>
  );
}

