import React, { Suspense } from "react";

export default function AssetsLayout({ children }: { children: React.ReactNode}) {
  return (
      <>
        <Suspense fallback={<div className="p-4">Loading...</div>}>
          <main>{children}</main>
        </Suspense>
      </>
  );
}
