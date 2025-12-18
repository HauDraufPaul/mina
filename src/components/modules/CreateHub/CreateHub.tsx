import Card from "../../ui/Card";

export default function CreateHub() {
  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          Create Hub
        </h1>
        <p className="text-gray-400">Creative coding and experimentation</p>
      </div>
      <Card title="Creative Playground">
        <p className="text-gray-400">Creative tools coming soon...</p>
      </Card>
    </div>
  );
}

