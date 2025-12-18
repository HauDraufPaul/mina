import Card from "../../ui/Card";

export default function SecurityCenter() {
  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          Security Center
        </h1>
        <p className="text-gray-400">Authentication and access control</p>
      </div>
      <Card title="Security Dashboard">
        <p className="text-gray-400">Security features coming soon...</p>
      </Card>
    </div>
  );
}

