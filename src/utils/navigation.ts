// Navigation utility for command bar
// This will be used by commands to navigate

let navigateFunction: ((path: string) => void) | null = null;

export function setNavigateFunction(fn: (path: string) => void) {
  navigateFunction = fn;
}

export async function navigate(path: string) {
  if (!navigateFunction) {
    // Fallback: use window.location if navigate function not set
    if (path.startsWith("/")) {
      window.location.href = path;
    } else {
      window.location.href = `/${path}`;
    }
    return;
  }
  
  // Normalize path
  const normalizedPath = path.startsWith("/") ? path : `/${path}`;
  navigateFunction(normalizedPath);
}

