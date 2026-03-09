import { FC } from "react";

export const KeySignaturePropertiesSection: FC = () => {
  return (
    <div
      className="flex min-w-max gap-0.5"
      data-testid="key-signature-properties-section"
    >
      <section className="flex min-w-[18rem] flex-col bg-neutral-900">
        <h3 className="text-xs text-white p-3 bg-neutral-800">Key signature</h3>
        <div
          className="min-h-[7.5rem] bg-neutral-900"
          data-testid="key-signature-properties-blank"
        />
      </section>
    </div>
  );
};
