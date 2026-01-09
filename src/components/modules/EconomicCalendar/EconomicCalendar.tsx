import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { Calendar, TrendingUp, TrendingDown, Filter, Plus } from "lucide-react";
import Modal from "@/components/ui/Modal";
import { useErrorHandler } from "@/utils/errorHandler";

interface EconomicEvent {
  id: number;
  name: string;
  country: string;
  event_type: string;
  scheduled_at: number;
  actual_value: number | null;
  forecast_value: number | null;
  previous_value: number | null;
  impact_score: number;
  created_at: number;
  updated_at: number;
}

export default function EconomicCalendar() {
  const [events, setEvents] = useState<EconomicEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedCountry, setSelectedCountry] = useState<string | null>(null);
  const [selectedType, setSelectedType] = useState<string | null>(null);
  const [selectedEvent, setSelectedEvent] = useState<EconomicEvent | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newEventName, setNewEventName] = useState("");
  const [newEventCountry, setNewEventCountry] = useState("US");
  const [newEventType, setNewEventType] = useState("GDP");
  const [newEventScheduled, setNewEventScheduled] = useState("");
  const [newEventForecast, setNewEventForecast] = useState("");
  const [newEventPrevious, setNewEventPrevious] = useState("");
  const errorHandler = useErrorHandler();

  useEffect(() => {
    loadEvents();
  }, [selectedCountry, selectedType]);

  const loadEvents = async () => {
    try {
      setLoading(true);
      const now = Math.floor(Date.now() / 1000);
      const fromTs = now - 7 * 24 * 3600; // 7 days ago
      const toTs = now + 30 * 24 * 3600; // 30 days ahead

      const result = await invoke<EconomicEvent[]>("list_economic_events", {
        fromTs,
        toTs,
        country: selectedCountry,
        eventType: selectedType,
      });

      setEvents(result);
    } catch (err) {
      errorHandler.showError("Failed to load events", err);
    } finally {
      setLoading(false);
    }
  };

  const countries = useMemo(() => {
    const unique = new Set(events.map((e) => e.country));
    return Array.from(unique).sort();
  }, [events]);

  const eventTypes = useMemo(() => {
    const unique = new Set(events.map((e) => e.event_type));
    return Array.from(unique).sort();
  }, [events]);

  const upcomingEvents = useMemo(() => {
    const now = Math.floor(Date.now() / 1000);
    return events
      .filter((e) => e.scheduled_at >= now && e.actual_value === null)
      .sort((a, b) => a.scheduled_at - b.scheduled_at)
      .slice(0, 10);
  }, [events]);

  const getImpactColor = (score: number) => {
    if (score >= 0.7) return "text-neon-red";
    if (score >= 0.4) return "text-neon-amber";
    return "text-neon-cyan";
  };

  const getImpactLabel = (score: number) => {
    if (score >= 0.7) return "High";
    if (score >= 0.4) return "Medium";
    return "Low";
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-200">Economic Calendar</h2>
          <p className="text-sm text-gray-400">Track economic events and their market impact</p>
        </div>
        <Button variant="primary" onClick={() => setShowCreateModal(true)}>
          <Plus className="w-4 h-4 mr-2" />
          Add Event
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Card title="Filters" subtitle="Filter events">
          <div className="space-y-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">Country</label>
              <select
                value={selectedCountry || ""}
                onChange={(e) => setSelectedCountry(e.target.value || null)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white focus:outline-none focus:border-neon-cyan"
              >
                <option value="">All Countries</option>
                {countries.map((country) => (
                  <option key={country} value={country}>
                    {country}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-2">Event Type</label>
              <select
                value={selectedType || ""}
                onChange={(e) => setSelectedType(e.target.value || null)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white focus:outline-none focus:border-neon-cyan"
              >
                <option value="">All Types</option>
                {eventTypes.map((type) => (
                  <option key={type} value={type}>
                    {type}
                  </option>
                ))}
              </select>
            </div>
            <Button
              variant="secondary"
              onClick={() => {
                setSelectedCountry(null);
                setSelectedType(null);
              }}
              className="w-full"
            >
              <Filter className="w-4 h-4 mr-2" />
              Clear Filters
            </Button>
          </div>
        </Card>

        <Card title="Upcoming Events" subtitle={`${upcomingEvents.length} events`}>
          <div className="space-y-2 max-h-96 overflow-y-auto">
            {upcomingEvents.map((event) => (
              <button
                key={event.id}
                onClick={() => setSelectedEvent(event)}
                className="w-full text-left p-3 bg-white/5 border border-white/10 rounded hover:border-white/20 transition-colors"
              >
                <div className="flex items-center justify-between mb-1">
                  <div className="font-semibold text-sm">{event.name}</div>
                  <div className={`text-xs font-semibold ${getImpactColor(event.impact_score)}`}>
                    {getImpactLabel(event.impact_score)}
                  </div>
                </div>
                <div className="text-xs text-gray-400">
                  {event.country} • {event.event_type}
                </div>
                <div className="text-xs text-gray-500 mt-1">
                  {new Date(event.scheduled_at * 1000).toLocaleString()}
                </div>
              </button>
            ))}
          </div>
        </Card>

        <Card
          title="Event Details"
          subtitle={selectedEvent ? selectedEvent.name : "Select an event"}
        >
          {selectedEvent ? (
            <div className="space-y-4">
              <div>
                <div className="text-sm text-gray-400 mb-1">Country</div>
                <div className="font-semibold">{selectedEvent.country}</div>
              </div>
              <div>
                <div className="text-sm text-gray-400 mb-1">Type</div>
                <div className="font-semibold">{selectedEvent.event_type}</div>
              </div>
              <div>
                <div className="text-sm text-gray-400 mb-1">Scheduled</div>
                <div className="font-semibold">
                  {new Date(selectedEvent.scheduled_at * 1000).toLocaleString()}
                </div>
              </div>
              <div>
                <div className="text-sm text-gray-400 mb-1">Impact Score</div>
                <div className={`text-lg font-bold ${getImpactColor(selectedEvent.impact_score)}`}>
                  {getImpactLabel(selectedEvent.impact_score)} ({selectedEvent.impact_score.toFixed(2)})
                </div>
              </div>
              {selectedEvent.forecast_value !== null && (
                <div>
                  <div className="text-sm text-gray-400 mb-1">Forecast</div>
                  <div className="font-semibold">{selectedEvent.forecast_value}</div>
                </div>
              )}
              {selectedEvent.previous_value !== null && (
                <div>
                  <div className="text-sm text-gray-400 mb-1">Previous</div>
                  <div className="font-semibold">{selectedEvent.previous_value}</div>
                </div>
              )}
              {selectedEvent.actual_value !== null && (
                <div>
                  <div className="text-sm text-gray-400 mb-1">Actual</div>
                  <div className="font-semibold text-neon-cyan">{selectedEvent.actual_value}</div>
                </div>
              )}
            </div>
          ) : (
            <div className="text-center py-8 text-gray-400">
              <Calendar className="w-12 h-12 mx-auto mb-2 text-gray-500" />
              <p>Select an event to view details</p>
            </div>
          )}
        </Card>
      </div>

      <Modal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        title="Add Economic Event"
      >
        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-2">Event Name</label>
            <input
              type="text"
              value={newEventName}
              onChange={(e) => setNewEventName(e.target.value)}
              placeholder="GDP Growth Rate"
              className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">Country</label>
            <input
              type="text"
              value={newEventCountry}
              onChange={(e) => setNewEventCountry(e.target.value)}
              placeholder="US"
              className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">Event Type</label>
            <select
              value={newEventType}
              onChange={(e) => setNewEventType(e.target.value)}
              className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white focus:outline-none focus:border-neon-cyan"
            >
              <option value="GDP">GDP</option>
              <option value="Interest Rate">Interest Rate</option>
              <option value="FOMC">FOMC</option>
              <option value="CPI">CPI</option>
              <option value="Inflation">Inflation</option>
              <option value="Unemployment">Unemployment</option>
              <option value="Retail Sales">Retail Sales</option>
              <option value="Manufacturing">Manufacturing</option>
              <option value="Other">Other</option>
            </select>
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">Scheduled Date & Time</label>
            <input
              type="datetime-local"
              value={newEventScheduled}
              onChange={(e) => setNewEventScheduled(e.target.value)}
              className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">Forecast Value (Optional)</label>
            <input
              type="number"
              step="0.01"
              value={newEventForecast}
              onChange={(e) => setNewEventForecast(e.target.value)}
              placeholder="2.5"
              className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">Previous Value (Optional)</label>
            <input
              type="number"
              step="0.01"
              value={newEventPrevious}
              onChange={(e) => setNewEventPrevious(e.target.value)}
              placeholder="2.3"
              className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan"
            />
          </div>
          <div className="flex justify-end gap-2">
            <Button variant="secondary" onClick={() => setShowCreateModal(false)}>
              Cancel
            </Button>
            <Button
              variant="primary"
              onClick={async () => {
                if (!newEventName.trim() || !newEventScheduled) {
                  errorHandler.showError("Please fill in required fields", new Error("Name and scheduled date are required"));
                  return;
                }

                try {
                  const scheduledAt = Math.floor(new Date(newEventScheduled).getTime() / 1000);
                  const forecast = newEventForecast ? parseFloat(newEventForecast) : null;
                  const previous = newEventPrevious ? parseFloat(newEventPrevious) : null;

                  await invoke("create_economic_event", {
                    name: newEventName.trim(),
                    country: newEventCountry.trim(),
                    eventType: newEventType,
                    scheduledAt,
                    forecastValue: forecast,
                    previousValue: previous,
                  });

                  setNewEventName("");
                  setNewEventCountry("US");
                  setNewEventType("GDP");
                  setNewEventScheduled("");
                  setNewEventForecast("");
                  setNewEventPrevious("");
                  setShowCreateModal(false);
                  await loadEvents();
                } catch (err) {
                  errorHandler.showError("Failed to create event", err);
                }
              }}
            >
              Create
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
