import MintForm from "../../components/mint-form";

export default function CreatePage() {
  return (
    <div className="mx-auto max-w-2xl px-4 py-10 sm:px-6 lg:px-8">
      {/* Header */}
      <div className="mb-10 text-center animate-fade-in">
        <h1 className="text-3xl font-bold text-foreground">Create an NFT</h1>
        <p className="mt-2 text-sm text-muted">
          Upload your artwork, set attributes, and mint directly to SolClone
        </p>
      </div>

      {/* Form */}
      <MintForm />
    </div>
  );
}
