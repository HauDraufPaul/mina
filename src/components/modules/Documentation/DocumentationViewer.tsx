import { useLocation } from "react-router-dom";
import DocusaurusFrame from "./DocusaurusFrame";

/**
 * Documentation Viewer - Embeds Docusaurus documentation
 * In development: connects to Docusaurus dev server (port 3000)
 * In production: serves from built static files
 */
export default function DocumentationViewer() {
  const location = useLocation();
  
  // Extract the path from the current route
  // /docs -> /, /docs/modules/automation-circuit -> /modules/automation-circuit
  let docusaurusPath = location.pathname.replace(/^\/docs/, "") || "/";
  
  // Docusaurus expects paths without trailing slashes (except root)
  if (docusaurusPath !== "/" && docusaurusPath.endsWith("/")) {
    docusaurusPath = docusaurusPath.slice(0, -1);
  }

  return (
    <div className="w-full h-full">
      <DocusaurusFrame path={docusaurusPath} />
    </div>
  );
}
