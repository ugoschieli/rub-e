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
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { handleGetAllProjects, handleAddAsset } from "@/components/services";
import * as React from "react";

export function AssetAddCard({ onClose }: { onClose: () => void }) {
  const [projects, setProjects] = React.useState<
    { name: string; id: number }[]
  >([]);
  const [assetName, setAssetName] = React.useState("");
  const [selectedProjectId, setSelectedProject] = React.useState<number>();
  const [isSuccess, setIsSuccess] = React.useState(false);

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

  const handleCreateAsset = async (e: React.MouseEvent<HTMLButtonElement>) => {
    e.preventDefault();
    if (!assetName || !selectedProjectId) {
      alert("Please fill all fields!");
      return;
    }

    const newAsset = {
      name: assetName,
      project_name: selectedProjectId,
      created_at: new Date().toISOString(),
    };

    handleAddAsset(assetName, selectedProjectId);
    setIsSuccess(true);
    setTimeout(() => {
      setIsSuccess(false);
      onClose();
    }, 2000);
  };

  return (
    <Card className="w-full max-w-sm">
      <CardHeader>
        <CardTitle>Created asset</CardTitle>
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
          </div>
        </form>
      </CardContent>
      <CardFooter className="flex-col gap-2">
        <Button type="submit" className="w-full" onClick={handleCreateAsset}>
          Create Asset
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
