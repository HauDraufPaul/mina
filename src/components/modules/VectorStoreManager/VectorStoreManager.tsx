import Card from "../../ui/Card";

export default function VectorStoreManager() {
  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          Vector Store Manager
        </h1>
        <p className="text-gray-400">Vector embeddings and semantic search</p>
      </div>
      <Card title="Vector Store">
        <p className="text-gray-400">Vector store features coming soon...</p>
      </Card>
    </div>
  );
}

