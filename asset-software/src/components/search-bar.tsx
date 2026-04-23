"use client";

import React, { useState, useEffect, useRef } from "react";
import { Input } from "@/components/ui/input";
import { useData } from "@/context/data-context";
import { Asset, Project, Category } from "@/types/types";
import Link from "next/link";

const SearchBar = () => {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<any[]>([]);
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const { assets, projects, categories } = useData();

  useEffect(() => {
    if (query.trim() === "") {
      setResults([]);
      setIsDropdownOpen(false);
      return;
    }

    const filteredAssets = assets
      .filter((asset) => asset.name.toLowerCase().includes(query.toLowerCase()))
      .map((asset) => ({ ...asset, type: "asset" }));

    const filteredProjects = projects
      .filter((project) =>
        project.name.toLowerCase().includes(query.toLowerCase()),
      )
      .map((project) => ({ ...project, type: "project" }));

    const filteredCategories = categories
      .filter((category) =>
        category.name.toLowerCase().includes(query.toLowerCase()),
      )
      .map((category) => ({ ...category, type: "category" }));

    setResults([
      ...filteredAssets,
      ...filteredProjects,
      ...filteredCategories,
    ]);
    setIsDropdownOpen(true);
  }, [query, assets, projects, categories]);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setIsDropdownOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  return (
    <div className="relative w-full max-w-lg" ref={dropdownRef}>
      <Input
        type="text"
        placeholder="Search assets, projects, categories..."
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        className="w-full"
      />
      {isDropdownOpen && results.length > 0 && (
        <div className="absolute top-full left-0 w-full bg-popover border border-border rounded-md mt-1 shadow-lg z-50 overflow-hidden">
          <ul className="max-h-60 overflow-y-auto">
            {results.map((result, index) => (
              <li
                key={`${result.type}-${result.id}-${index}`}
                className="hover:bg-accent hover:text-accent-foreground transition-colors"
                onClick={() => setIsDropdownOpen(false)}
              >
                <Link
                  href={
                    result.type === "asset"
                      ? `/editor/${result.id}`
                      : result.type === "project"
                        ? `/projects/${result.id}`
                        : `/assets/${result.id}`
                  }
                  className="block px-4 py-2"
                >
                  <div className="flex justify-between items-center">
                    <span className="text-sm font-medium">
                      {result.name}
                    </span>
                    <span className="text-xs text-muted-foreground uppercase">
                      {result.type}
                    </span>
                  </div>
                </Link>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
};

export default SearchBar;
