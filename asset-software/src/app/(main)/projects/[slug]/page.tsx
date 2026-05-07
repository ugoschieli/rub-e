import React from "react";
import ProjectClient from "./project-client";
import data_projects from "@/../config/data_projects.json";

export async function generateStaticParams() {
  return data_projects.map((project) => ({
    slug: project.id.toString(),
  }));
}

export default function Page({ params }: { params: Promise<{ slug: string }> }) {
  return <ProjectClient params={params} />;
}
