import { useEffect, useRef, useState } from "react";
import { useLocation } from "react-router-dom";
import Card from "../../ui/Card";

interface DocusaurusFrameProps {
  path?: string;
}

export default function DocusaurusFrame({ path }: DocusaurusFrameProps) {
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const location = useLocation();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Determine the Docusaurus URL based on environment
  const getDocsUrl = () => {
    // In development, Docusaurus runs on port 3000
    if (import.meta.env.DEV) {
      const docusaurusPath = path || location.pathname.replace("/docs", "") || "/";
      return `http://localhost:3000/docs${docusaurusPath}`;
    }
    
    // In production, serve from built static files
    // Docusaurus build output should be in dist/docs/
    const docusaurusPath = path || location.pathname.replace("/docs", "") || "/";
    return `/docs${docusaurusPath}`;
  };

  useEffect(() => {
    const iframe = iframeRef.current;
    if (!iframe) return;

    const handleLoad = () => {
      setLoading(false);
      setError(null);
    };

    const handleError = () => {
      setLoading(false);
      setError("Failed to load documentation. Make sure Docusaurus is running in development mode.");
    };

    iframe.addEventListener("load", handleLoad);
    iframe.addEventListener("error", handleError);

    // Set the source
    iframe.src = getDocsUrl();

    return () => {
      iframe.removeEventListener("load", handleLoad);
      iframe.removeEventListener("error", handleError);
    };
  }, [location.pathname, path]);

  if (error) {
    return (
      <Card>
        <div className="p-8 text-center">
          <p className="text-red-400 mb-4">{error}</p>
          <p className="text-gray-400 text-sm">
            In development, run <code className="bg-white/5 px-2 py-1 rounded">npm run docs:dev</code> to start Docusaurus
          </p>
        </div>
      </Card>
    );
  }

  return (
    <div className="relative w-full h-full min-h-[600px]">
      {loading && (
        <div className="absolute inset-0 flex items-center justify-center bg-black/50 z-10">
          <div className="text-center">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-neon-cyan mx-auto mb-4"></div>
            <p className="text-gray-400">Loading documentation...</p>
          </div>
        </div>
      )}
      <iframe
        ref={iframeRef}
        className="w-full h-full border-0 rounded-lg"
        style={{ minHeight: "600px" }}
        title="Docusaurus Documentation"
        allow="fullscreen"
      />
    </div>
  );
}
