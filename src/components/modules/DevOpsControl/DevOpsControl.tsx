import Card from "../../ui/Card";

export default function DevOpsControl() {
  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          DevOps Control
        </h1>
        <p className="text-gray-400">Prometheus and service monitoring</p>
      </div>
      <Card title="DevOps Dashboard">
        <p className="text-gray-400">DevOps features coming soon...</p>
      </Card>
    </div>
  );
}

