import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { handleGetAllProjects, handleImportAsset } from "@/components/services";
import * as React from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useData } from "@/context/data-context";

export function AssetImportCard({ onClose }: { onClose: () => void }) {
  const { refreshData } = useData();
  const [projects, setProjects] = React.useState<{ name: string; id: number }[]>([]);
  const [assetName, setAssetName] = React.useState("");
  const [selectedProjectId, setSelectedProject] = React.useState<number>();
  const [selectedFile, setSelectedFile] = React.useState<string | null>(null);
  const [isSuccess, setIsSuccess] = React.useState(false);
  const fileInputRef = React.useRef<HTMLInputElement>(null);

  React.useEffect(() => {
    async function fetchProjects() {
      try {
        const projects = (await handleGetAllProjects()) as {
          name: string;
          id: number;
        }[];
        setProjects(projects);
      } catch (err) {
        console.error("Failed to fetch projects:", err);
      }
    }
    fetchProjects();
  }, []);

  const handleFileSelect = async () => {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "Model",
          extensions: ["model"],
        },
      ],
    });

    if (typeof selected === "string") {
      setSelectedFile(selected);
    }
  };

  const handleAssetImport = async (e: React.MouseEvent<HTMLButtonElement>) => {
    e.preventDefault();

    if (!assetName || !selectedProjectId || !selectedFile) {
      alert("Please fill all fields!");
      return;
    }

    try {
      await handleImportAsset(assetName, selectedProjectId, selectedFile);

      await refreshData();

      setIsSuccess(true);

      setTimeout(() => {
        setIsSuccess(false);
        onClose();
      }, 2000);

    } catch (err) {
      alert("Failed to import asset");
    }
  };

  return (
    <Card className="w-full max-w-sm">
      <CardHeader>
        <CardTitle>Import asset</CardTitle>
        <CardDescription>Enter your asset details below</CardDescription>
      </CardHeader>
      <CardContent>
        <form>
          <div className="grid w-full items-center gap-4">
            {/* Name asset */}
            <div className="grid w-full items-center gap-2">
              <Label htmlFor="name">Name</Label>
              <Input
                id="name"
                placeholder="Asset name"
                value={assetName}
                onChange={(e) => setAssetName(e.target.value)}
              />
            </div>
            {/* List project */}
            <div className="grid w-full items-center gap-2">
              <Label htmlFor="project">Project</Label>
              <Select
                value={selectedProjectId?.toString()}
                onValueChange={(val) => setSelectedProject(Number(val))}
              >
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Select a project" />
                </SelectTrigger>
                <SelectContent>
                  {projects?.map((project) => (
                    <SelectItem key={project.id} value={project.id.toString()}>
                      {project.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            {/* Asset Origine */}
            <div className="grid w-full items-center gap-2">
              <Label htmlFor="asset-origine">Origin Asset</Label>
              <div className="flex gap-2">
                <Input
                  id="asset-origine"
                  placeholder="No .model file selected"
                  value={selectedFile || ""}
                  disabled
                  className="flex-1"
                />
                <Button
                  type="button"
                  variant="outline"
                  onClick={handleFileSelect}
                >
                  Select Asset
                </Button>
              </div>
            </div>
          </div>
        </form>
      </CardContent>
      <CardFooter className="flex-col gap-2">
        <Button type="submit" className="w-full" onClick={handleAssetImport}>
          Import asset
        </Button>
        <Button
          variant="outline"
          className="w-full"
          type="button"
          onClick={onClose}
        >
          Cancel
        </Button>
      </CardFooter>
    </Card>
  );
}