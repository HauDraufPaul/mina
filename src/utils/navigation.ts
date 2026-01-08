// Navigation utility for command bar
// This will be used by commands to navigate

let navigateFunction: ((path: string) => void) | null = null;

export function setNavigateFunction(fn: (path: string) => void) {
  navigateFunction = fn;
}

export function navigate(path: string) {
  // Normalize path - ensure it starts with /
  const normalizedPath = path.startsWith("/") ? path : `/${path}`;
  
  if (!navigateFunction) {
    // Fallback: use window.location if navigate function not set
    console.warn("Navigate function not set, using window.location for:", normalizedPath);
    window.location.href = normalizedPath;
    return;
  }
  
  console.log("Navigating to:", normalizedPath);
  try {
    navigateFunction(normalizedPath);
  } catch (error) {
    console.error("Navigation error:", error);
    // Fallback to window.location
    window.location.href = normalizedPath;
  }
}

