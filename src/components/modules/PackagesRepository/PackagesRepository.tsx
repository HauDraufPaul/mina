import Card from "../../ui/Card";

export default function PackagesRepository() {
  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          Packages Repository
        </h1>
        <p className="text-gray-400">Homebrew package management</p>
      </div>
      <Card title="Package Manager">
        <p className="text-gray-400">Package management features coming soon...</p>
      </Card>
    </div>
  );
}

