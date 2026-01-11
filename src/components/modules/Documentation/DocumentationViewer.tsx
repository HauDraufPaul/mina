import { useEffect, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { ArrowLeft } from "lucide-react";

/**
 * Native Docusaurus Integration
 * Loads and renders Docusaurus-built HTML directly in the app
 */
export default function DocumentationViewer() {
  const location = useLocation();
  const navigate = useNavigate();
  const [htmlContent, setHtmlContent] = useState<string>("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const loadDocumentation = async () => {
      setLoading(true);
      setError(null);

      // Get the Docusaurus path
      let docPath = location.pathname.replace(/^\/docs/, "") || "/";
      if (docPath !== "/" && !docPath.endsWith("/")) {
        docPath = docPath + "/";
      }
      if (!docPath.startsWith("/")) {
        docPath = "/" + docPath;
      }

      // Build the URL to the Docusaurus HTML file
      const url = `/docs${docPath}index.html`;

      try {
        console.log("Loading Docusaurus from:", url);
        const response = await fetch(url);
        
        if (!response.ok) {
          throw new Error(`Failed to load: ${response.status}`);
        }

        const html = await response.text();
        console.log("Loaded HTML, length:", html.length);
        
        // Extract the main content and inject it
        const parser = new DOMParser();
        const doc = parser.parseFromString(html, "text/html");
        
        // Get the main wrapper - Docusaurus uses #__docusaurus
        let docusaurusRoot = doc.getElementById("__docusaurus");
        if (!docusaurusRoot) {
          // Fallback: try to find it by class or tag
          docusaurusRoot = doc.querySelector("#__docusaurus") || doc.body;
          console.warn("Using fallback root element");
        }

        // Remove navbar and footer, keep main content
        const navbar = docusaurusRoot.querySelector("nav");
        const footer = docusaurusRoot.querySelector("footer");
        if (navbar) {
          console.log("Removing navbar");
          navbar.remove();
        }
        if (footer) {
          console.log("Removing footer");
          footer.remove();
        }

        // Get the main content
        let main = docusaurusRoot.querySelector("main");
        if (!main) {
          // Fallback: try to find content in body
          main = doc.querySelector("main") || docusaurusRoot.querySelector(".main-wrapper") || docusaurusRoot;
          console.warn("Using fallback main element");
        }
        
        console.log("Found main element:", !!main, "Type:", main?.tagName);

        // Process links to work with React Router
        const links = main.querySelectorAll("a");
        links.forEach((link) => {
          const href = link.getAttribute("href");
          if (href && (href.startsWith("/docs/") || href.startsWith("/intro") || href.startsWith("/getting-started/") || href.startsWith("/modules/") || href.startsWith("/api/") || href.startsWith("/guides/"))) {
            link.setAttribute("data-doc-link", "true");
            link.addEventListener("click", (e) => {
              e.preventDefault();
              const targetPath = href.startsWith("/") ? href : `/docs${href}`;
              navigate(targetPath);
            });
          }
        });

        // Fix asset paths
        const assets = main.querySelectorAll("[src], [href]");
        assets.forEach((asset) => {
          const src = asset.getAttribute("src") || asset.getAttribute("href");
          if (src && src.startsWith("/docs/")) {
            // Keep as is - Vite will serve it
          } else if (src && src.startsWith("/")) {
            // Make it relative to /docs
            asset.setAttribute(src.includes("src") ? "src" : "href", `/docs${src}`);
          }
        });

        // Inject Docusaurus CSS
        const stylesheets = doc.querySelectorAll("link[rel='stylesheet']");
        stylesheets.forEach((link) => {
          const href = link.getAttribute("href");
          if (href && !document.querySelector(`link[href="${href}"]`)) {
            const linkEl = document.createElement("link");
            linkEl.rel = "stylesheet";
            linkEl.href = href.startsWith("/") ? href : `/docs${href}`;
            document.head.appendChild(linkEl);
          }
        });

        // Inject Docusaurus JS (but only once)
        if (!window.__docusaurus_loaded) {
          const scripts = doc.querySelectorAll("script[src]");
          scripts.forEach((script) => {
            const src = script.getAttribute("src");
            if (src && !document.querySelector(`script[src="${src}"]`)) {
              const scriptEl = document.createElement("script");
              scriptEl.src = src.startsWith("/") ? src : `/docs${src}`;
              scriptEl.defer = true;
              document.head.appendChild(scriptEl);
            }
          });
          window.__docusaurus_loaded = true;
        }

        const content = main.outerHTML || main.innerHTML;
        console.log("Extracted content, length:", content.length);
        console.log("First 500 chars:", content.substring(0, 500));
        
        if (content.length < 50) {
          throw new Error(`Content too short: ${content.length} characters. HTML structure may have changed.`);
        }
        
        setHtmlContent(content);
      } catch (err) {
        console.error("Failed to load documentation:", err);
        setError(`Failed to load documentation: ${err instanceof Error ? err.message : String(err)}`);
      } finally {
        setLoading(false);
      }
    };

    loadDocumentation();
  }, [location.pathname, navigate]);

  // Handle link clicks after content is rendered
  useEffect(() => {
    if (!htmlContent) return;

    const handleClick = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      const link = target.closest("a[data-doc-link='true']");
      
      if (link) {
        e.preventDefault();
        const href = link.getAttribute("href");
        if (href) {
          navigate(href.startsWith("/") ? href : `/docs${href}`);
        }
      }
    };
    
    // Wait for DOM to update
    const timeout = setTimeout(() => {
      const container = document.getElementById("docusaurus-container");
      if (container) {
        container.addEventListener("click", handleClick);
      }
    }, 100);
    
    return () => {
      clearTimeout(timeout);
      const container = document.getElementById("docusaurus-container");
      if (container) {
        container.removeEventListener("click", handleClick);
      }
    };
  }, [navigate, htmlContent]);

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-neon-cyan mx-auto mb-4"></div>
          <p className="text-gray-400">Loading documentation...</p>
        </div>
      </div>
    );
  }

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

  if (!htmlContent) {
    return (
      <Card>
        <div className="p-8 text-center">
          <p className="text-gray-400">No content loaded.</p>
        </div>
      </Card>
    );
  }

  return (
    <div className="w-full h-full overflow-auto bg-gray-950">
      <div 
        id="docusaurus-container" 
        dangerouslySetInnerHTML={{ __html: htmlContent }}
        style={{ minHeight: "400px" }}
      />
      <style>{`
        #docusaurus-container {
          background: transparent;
          color: #e5e5e5;
        }
        #docusaurus-container * {
          box-sizing: border-box;
        }
      `}</style>
      {/* Debug info in dev */}
      {import.meta.env.DEV && (
        <div className="fixed bottom-4 right-4 bg-black/80 p-2 text-xs text-gray-400 rounded z-50">
          <div>Content length: {htmlContent.length}</div>
          <div>Loading: {loading ? "yes" : "no"}</div>
          <div>Error: {error ? "yes" : "no"}</div>
        </div>
      )}
    </div>
  );
}
