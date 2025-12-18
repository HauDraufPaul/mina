import Card from "../../ui/Card";

export default function MigrationManager() {
  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          Migration Manager
        </h1>
        <p className="text-gray-400">Database migrations</p>
      </div>
      <Card title="Migrations">
        <p className="text-gray-400">Migration management coming soon...</p>
      </Card>
    </div>
  );
}

