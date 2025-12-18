import Card from "../../ui/Card";

export default function TestingCenter() {
  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          Testing Center
        </h1>
        <p className="text-gray-400">Test suite and coverage</p>
      </div>
      <Card title="Test Dashboard">
        <p className="text-gray-400">Testing features coming soon...</p>
      </Card>
    </div>
  );
}

