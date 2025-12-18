import Card from "../../ui/Card";

export default function ErrorDashboard() {
  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          Error Dashboard
        </h1>
        <p className="text-gray-400">Error tracking and analysis</p>
      </div>
      <Card title="Errors">
        <p className="text-gray-400">Error tracking coming soon...</p>
      </Card>
    </div>
  );
}

