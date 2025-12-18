import Card from "../../ui/Card";

export default function NetworkConstellation() {
  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          Network Constellation
        </h1>
        <p className="text-gray-400">Network monitoring and analysis</p>
      </div>
      <Card title="Network Connections">
        <p className="text-gray-400">Network monitoring features coming soon...</p>
      </Card>
    </div>
  );
}

