import { useToast } from "@/components/ui/Toast";

/**
 * Error handling utility for consistent error UI feedback
 * Replaces alert() and console.error() patterns
 */

export interface ErrorHandler {
  showError: (message: string, error?: unknown) => void;
  showSuccess: (message: string) => void;
  showInfo: (message: string) => void;
  showWarning: (message: string) => void;
}

/**
 * Creates an error handler using the toast system
 * Use this in React components to handle errors properly
 */
export function useErrorHandler(): ErrorHandler {
  const toast = useToast();

  const showError = (message: string, error?: unknown) => {
    const errorMessage = error instanceof Error ? error.message : String(error);
    const fullMessage = error ? `${message}: ${errorMessage}` : message;
    toast.error(fullMessage, 7000); // Show errors longer
    console.error(message, error);
  };

  const showSuccess = (message: string) => {
    toast.success(message);
  };

  const showInfo = (message: string) => {
    toast.info(message);
  };

  const showWarning = (message: string) => {
    toast.warning(message);
  };

  return {
    showError,
    showSuccess,
    showInfo,
    showWarning,
  };
}

/**
 * Validates input and shows error if invalid
 */
export function validateInput(
  value: string,
  rules: {
    required?: boolean;
    minLength?: number;
    maxLength?: number;
    pattern?: RegExp;
    patternMessage?: string;
  },
  errorHandler: ErrorHandler
): boolean {
  if (rules.required && !value.trim()) {
    errorHandler.showError("This field is required");
    return false;
  }

  if (rules.minLength && value.length < rules.minLength) {
    errorHandler.showError(`Minimum length is ${rules.minLength} characters`);
    return false;
  }

  if (rules.maxLength && value.length > rules.maxLength) {
    errorHandler.showError(`Maximum length is ${rules.maxLength} characters`);
    return false;
  }

  if (rules.pattern && !rules.pattern.test(value)) {
    errorHandler.showError(rules.patternMessage || "Invalid format");
    return false;
  }

  return true;
}

