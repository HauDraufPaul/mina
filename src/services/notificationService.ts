import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface NotificationOptions {
  title: string;
  body: string;
  icon?: string;
  sound?: string;
  tag?: string;
  data?: Record<string, any>;
}

class NotificationService {
  private permissionGranted: boolean = false;

  async requestPermission(): Promise<boolean> {
    if ("Notification" in window) {
      if (Notification.permission === "granted") {
        this.permissionGranted = true;
        return true;
      }
      
      if (Notification.permission !== "denied") {
        const permission = await Notification.requestPermission();
        this.permissionGranted = permission === "granted";
        return this.permissionGranted;
      }
    }
    
    return false;
  }

  async sendNotification(options: NotificationOptions): Promise<void> {
    // Request permission if not already granted
    if (!this.permissionGranted) {
      await this.requestPermission();
    }

    // Use native browser notifications if available
    if ("Notification" in window && Notification.permission === "granted") {
      const notification = new Notification(options.title, {
        body: options.body,
        icon: options.icon || "/icon.png",
        tag: options.tag,
        data: options.data,
      });

      // Auto-close after 5 seconds
      setTimeout(() => {
        notification.close();
      }, 5000);

      // Handle click
      notification.onclick = () => {
        window.focus();
        notification.close();
      };
    } else {
      // Fallback: use Tauri command
      await invoke("send_notification", {
        title: options.title,
        body: options.body,
        icon: options.icon,
        sound: options.sound,
        tag: options.tag,
        data: options.data,
      });
    }
  }

  async sendAlertNotification(alertId: number, title: string, message: string): Promise<void> {
    await this.sendNotification({
      title: `Alert: ${title}`,
      body: message,
      icon: "alert",
      tag: `alert-${alertId}`,
      data: { type: "alert", alertId },
    });
  }

  async sendPriceAlertNotification(
    ticker: string,
    condition: string,
    targetPrice: number,
    currentPrice: number
  ): Promise<void> {
    await this.sendNotification({
      title: `Price Alert: ${ticker}`,
      body: `${ticker} ${condition} $${targetPrice.toFixed(2)} (Current: $${currentPrice.toFixed(2)})`,
      icon: "price-alert",
      tag: `price-alert-${ticker}`,
      data: {
        type: "price_alert",
        ticker,
        condition,
        targetPrice,
        currentPrice,
      },
    });
  }

  setupEventListeners(): () => void {
    // Listen for desktop notification events from backend
    const unlistenAlert = listen("desktop-notification", (event) => {
      const payload = event.payload as NotificationOptions;
      this.sendNotification(payload);
    });

    const unlistenAlertNotification = listen("alert-notification", (event) => {
      const payload = event.payload as { id: number; title: string; message: string };
      this.sendAlertNotification(payload.id, payload.title, payload.message);
    });

    const unlistenPriceAlert = listen("price-alert-triggered", (event) => {
      const payload = event.payload as {
        ticker: string;
        condition: string;
        target_price: number;
        current_price: number;
      };
      this.sendPriceAlertNotification(
        payload.ticker,
        payload.condition,
        payload.target_price,
        payload.current_price
      );
    });

    // Return cleanup function
    return () => {
      unlistenAlert.then((fn) => fn());
      unlistenAlertNotification.then((fn) => fn());
      unlistenPriceAlert.then((fn) => fn());
    };
  }
}

export const notificationService = new NotificationService();

