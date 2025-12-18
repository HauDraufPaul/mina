import Card from "../../ui/Card";

export default function ConfigurationManager() {
  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          Configuration Manager
        </h1>
        <p className="text-gray-400">Application configuration</p>
      </div>
      <Card title="Configuration">
        <p className="text-gray-400">Configuration management coming soon...</p>
      </Card>
    </div>
  );
}

