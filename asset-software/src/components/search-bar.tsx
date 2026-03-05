"use client"

import { FormEvent, useState, useMemo, useRef, useEffect } from "react";
import { useRouter } from "next/navigation";
import dataAssets from "@/../config/data_assets.json";
import dataProjects from "@/../config/data_projects.json";
import dataCategories from "@/../config/data_categories.json";
import { ChevronDown, Search, Grid } from "lucide-react";

interface SearchResult {
    id: number;
    name: string;
}

type SearchSource = 'all' | 'category' | 'project' | 'asset';

export default function SearchBar() {
    const [query, setQuery] = useState("");
    const [isOpen, setIsOpen] = useState(false);
    const [source, setSource] = useState<SearchSource>('all');
    const [isSourceOpen, setIsSourceOpen] = useState(false);
    const router = useRouter();
    const sourceRef = useRef<HTMLDivElement>(null);

    // Fermer le menu de sélection de source au clic extérieur
    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (sourceRef.current && !sourceRef.current.contains(event.target as Node)) {
                setIsSourceOpen(false);
            }
        };
        document.addEventListener("mousedown", handleClickOutside);
        return () => document.removeEventListener("mousedown", handleClickOutside);
    }, []);

    // Filtrage des données selon la saisie et la source sélectionnée
    const filteredResults = useMemo(() => {
        if (!query.trim()) return { assets: [], projects: [], categories: [] };

        const search = query.toLowerCase();
        return {
            assets: (source === 'all' || source === 'asset')
                ? (dataAssets as SearchResult[]).filter(i => i.name.toLowerCase().includes(search)).slice(0, 5)
                : [],
            projects: (source === 'all' || source === 'project')
                ? (dataProjects as SearchResult[]).filter(i => i.name.toLowerCase().includes(search)).slice(0, 3)
                : [],
            categories: (source === 'all' || source === 'category')
                ? (dataCategories as SearchResult[]).filter(i => i.name.toLowerCase().includes(search)).slice(0, 3)
                : [],
        };
    }, [query, source]);

    const hasResults = filteredResults.assets.length > 0 ||
                      filteredResults.projects.length > 0 ||
                      filteredResults.categories.length > 0;

    const handleSubmit = (e: FormEvent<HTMLFormElement>) => {
        e.preventDefault();
        if (!query.trim()) return;

        let url = `/assets?search=${encodeURIComponent(query)}`;
        if (source !== 'all') {
            url += `&filter_type=${source}`;
        }

        router.push(url);
        setIsOpen(false);
    };

    const handleSelect = (item: SearchResult, type: 'asset' | 'project' | 'category') => {
        setIsOpen(false);
        setQuery(item.name);

        switch (type) {
            case 'asset':
                router.push(`/assets?search=${encodeURIComponent(item.name)}`);
                break;
            case 'project':
                router.push(`/projects/${item.id}`);
                break;
            case 'category':
                router.push(`/assets/${item.id}`);
                break;
        }
    };

    const sourceLabels: Record<SearchSource, string> = {
        all: "All sources",
        category: "Categories",
        project: "Projects",
        asset: "Assets"
    };

    return (
        <div className="relative max-w-2xl mx-auto">
            <form className="w-full" onSubmit={handleSubmit}>
                <div className="flex shadow-xs rounded-base -space-x-0.5">
                    <div className="relative" ref={sourceRef}>
                        <button
                            type="button"
                            onClick={() => setIsSourceOpen(!isSourceOpen)}
                            className="cursor-pointer inline-flex items-center shrink-0 z-20 text-body bg-neutral-secondary-medium box-border border border-default-medium hover:bg-neutral-tertiary-medium hover:text-heading font-medium leading-5 rounded-s-base text-sm px-4 py-2.5 focus:outline-none transition-colors min-w-[140px]"
                        >
                            <Grid className="w-4 h-4 me-1.5 " />
                            <span className="truncate">{sourceLabels[source]}</span>
                            <ChevronDown className={`ml-auto w-4 h-4 transition-transform ${isSourceOpen ? 'rotate-180' : ''}`} />
                        </button>

                        {isSourceOpen && (
                            <div className="absolute left-0 top-full mt-1 z-[110] w-48 bg-[#1a1a1a] border border-default-medium rounded-base shadow-lg overflow-hidden">
                                {(Object.keys(sourceLabels) as SearchSource[]).map((s) => (
                                    <button
                                        key={s}
                                        type="button"
                                        onClick={() => {
                                            setSource(s);
                                            setIsSourceOpen(false);
                                        }}
                                        className={`cursor-pointer hover:bg-slate-100 hover:text-black w-full text-left px-4 py-2 text-sm hover:bg-neutral-tertiary-medium transition-colors ${source === s ? 'text-brand font-bold' : 'text-body'}`}
                                    >
                                        {sourceLabels[s]}
                                    </button>
                                ))}
                            </div>
                        )}
                    </div>

                    <div className="relative w-full">
                        <input
                            type="search"
                            className="px-3 py-2.5 bg-neutral-secondary-medium border border-default-medium text-heading text-sm focus:ring-brand focus:border-brand block w-full placeholder:text-body outline-none"
                            placeholder={`Search ${source === 'all' ? 'assets, projects...' : sourceLabels[source].toLowerCase() + '...'}`}
                            value={query}
                            onChange={(e) => {
                                setQuery(e.target.value);
                                setIsOpen(true);
                            }}
                            onFocus={() => query.length > 0 && setIsOpen(true)}
                        />

                        {/* Dropdown de résultats */}
                        {isOpen && hasResults && (
                            <div className="absolute z-[100] bg-[#1a1a1a] w-full mt-1 border border-default-medium rounded-base shadow-lg overflow-hidden max-h-96 overflow-y-auto">
                                <div className="p-2 space-y-3">
                                    {filteredResults.assets.length > 0 && (
                                        <div>
                                            <header className="px-2 py-1 text-[10px] font-bold uppercase text-body opacity-60 tracking-wider">Assets</header>
                                            {filteredResults.assets.map(item => (
                                                <button key={`as-${item.id}`} type="button" onClick={() => handleSelect(item, 'asset')}
                                                        className="w-full text-left block px-2 py-2 hover:bg-neutral-tertiary-medium hover:text-heading rounded-md text-sm transition-colors">
                                                    {item.name}
                                                </button>
                                            ))}
                                        </div>
                                    )}

                                    {filteredResults.projects.length > 0 && (
                                        <div>
                                            <header className="px-2 py-1 text-[10px] font-bold uppercase text-body opacity-60 tracking-wider">Projects</header>
                                            {filteredResults.projects.map(item => (
                                                <button key={`pj-${item.id}`} type="button" onClick={() => handleSelect(item, 'project')}
                                                        className="w-full text-left block px-2 py-2 hover:bg-neutral-tertiary-medium hover:text-heading rounded-md text-sm transition-colors">
                                                    {item.name}
                                                </button>
                                            ))}
                                        </div>
                                    )}

                                    {filteredResults.categories.length > 0 && (
                                        <div>
                                            <header className="px-2 py-1 text-[10px] font-bold uppercase text-body opacity-60 tracking-wider">Categories</header>
                                            {filteredResults.categories.map(item => (
                                                <button key={`cat-${item.id}`} type="button" onClick={() => handleSelect(item, 'category')}
                                                        className="w-full text-left block px-2 py-2 hover:bg-neutral-tertiary-medium hover:text-heading rounded-md text-sm transition-colors">
                                                    {item.name}
                                                </button>
                                            ))}
                                        </div>
                                    )}
                                </div>
                            </div>
                        )}
                    </div>

                    <button type="submit"
                            className="inline-flex items-center text-white bg-brand hover:bg-brand-strong box-border border border-transparent focus:ring-4 focus:ring-brand-medium shadow-xs font-medium leading-5 rounded-e-base text-sm px-4 py-2.5 focus:outline-none transition-colors">
                        <Search className="w-4 h-4 me-1.5" />
                        Search
                    </button>
                </div>
            </form>

            {isOpen && (
                <div className="fixed inset-0 z-40" onClick={() => setIsOpen(false)}></div>
            )}
        </div>
    );
}