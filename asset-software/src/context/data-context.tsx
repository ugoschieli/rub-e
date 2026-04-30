"use client";

import React, { createContext, useContext, useState, useEffect, useCallback } from "react";
import { handleGetAllAssets, handleGetAllProjects, handleGetAllCategories } from "@/components/services";
import { Asset, Project, Category } from "@/types/types";

interface DataState {
  assets: Asset[];
  projects: Project[];
  categories: Category[];
  isLoading: boolean;
  refreshData: () => Promise<void>;
}

const DataContext = createContext<DataState | undefined>(undefined);

export function DataProvider({ children }: { children: React.ReactNode }) {
  const [assets, setAssets] = useState<Asset[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  const refreshData = useCallback(async () => {
    setIsLoading(true);
    try {
      const [allAssets, allProjects, allCategories] = await Promise.all([
        handleGetAllAssets(),
        handleGetAllProjects(),
        handleGetAllCategories()
      ]);
      
      setAssets(allAssets || []);
      setProjects(allProjects || []);
      setCategories((allCategories as Category[]) || []);
    } catch (error) {
      console.error("Failed to fetch data:", error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    refreshData();
  }, [refreshData]);

  return (
    <DataContext.Provider
      value={{
        assets,
        projects,
        categories,
        isLoading,
        refreshData
      }}
    >
      {children}
    </DataContext.Provider>
  );
}

export function useData() {
  const context = useContext(DataContext);
  if (context === undefined) {
    throw new Error("useData must be used within a DataProvider");
  }
  return context;
}
