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
    // Get the path from location or prop
    let docusaurusPath = path;
    if (!docusaurusPath) {
      // Extract path from location, removing /docs prefix
      docusaurusPath = location.pathname.replace(/^\/docs/, "") || "/";
    }
    
    // Ensure path starts with / and doesn't end with / (except root)
    if (docusaurusPath !== "/" && !docusaurusPath.endsWith("/")) {
      docusaurusPath = docusaurusPath + "/";
    }
    if (!docusaurusPath.startsWith("/")) {
      docusaurusPath = "/" + docusaurusPath;
    }
    
    // Always serve from built static files (both dev and production)
    // Docusaurus build output is in dist/docs/
    // Docusaurus uses /docs/ as baseUrl, so paths are relative to that
    // Vite will serve these files from the public/dist or dist directory
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
      setError("Documentation not found. Make sure docs are built: npm run docs:build");
    };

    // Set timeout for loading
    const timeout = setTimeout(() => {
      if (loading) {
        setError("Documentation is taking too long to load. Check if Docusaurus is running.");
      }
    }, 10000);

    iframe.addEventListener("load", handleLoad);
    iframe.addEventListener("error", handleError);

    // Set the source
    const docsUrl = getDocsUrl();
    iframe.src = docsUrl;

    return () => {
      clearTimeout(timeout);
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
            Run <code className="bg-white/5 px-2 py-1 rounded">npm run docs:build</code> to build documentation
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
