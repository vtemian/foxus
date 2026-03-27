import { useState } from "react";
import { CategoriesTab } from "@/components/settings/category-tab";
import { RulesTab } from "@/components/settings/rules-tab";
import { TabButton } from "@/components/settings/shared";
import type { SettingsViewProps } from "@/components/settings/types";
import { Typography } from "@/components/ui";
import type { UseSettingsReturn } from "@/hooks/use-settings";
import { useSettings } from "@/hooks/use-settings";

type Tab = "categories" | "rules";

const SettingsHeader = ({ onClose }: { onClose: () => void }) => (
  <div className="flex items-center justify-between">
    <Typography variant="h2">Settings</Typography>
    <button
      type="button"
      onClick={onClose}
      className="font-mono text-xs text-gray-400 hover:text-gray-600 px-2 py-1"
      aria-label="Close settings"
    >
      [X]
    </button>
  </div>
);

const TabNav = ({
  activeTab,
  onTabChange,
}: {
  activeTab: Tab;
  onTabChange: (tab: Tab) => void;
}) => (
  <div className="flex gap-2 border-b border-gray-250 pb-2">
    <TabButton active={activeTab === "categories"} onClick={() => onTabChange("categories")}>
      Categories
    </TabButton>
    <TabButton active={activeTab === "rules"} onClick={() => onTabChange("rules")}>
      Rules
    </TabButton>
  </div>
);

const TabContent = ({ activeTab, settings }: { activeTab: Tab; settings: UseSettingsReturn }) => (
  <>
    {activeTab === "categories" && (
      <CategoriesTab
        categories={settings.categories}
        createCategory={settings.createCategory}
        updateCategory={settings.updateCategory}
        deleteCategory={settings.deleteCategory}
      />
    )}
    {activeTab === "rules" && (
      <RulesTab
        rules={settings.rules}
        categories={settings.categories}
        createRule={settings.createRule}
        updateRule={settings.updateRule}
        deleteRule={settings.deleteRule}
      />
    )}
  </>
);

const SettingsView = ({ onClose }: SettingsViewProps) => {
  const [activeTab, setActiveTab] = useState<Tab>("categories");
  const settings = useSettings();

  if (settings.isLoading) {
    return (
      <div className="py-8">
        <Typography variant="body" color="muted" className="text-center">
          Loading settings...
        </Typography>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <SettingsHeader onClose={onClose} />
      <TabNav activeTab={activeTab} onTabChange={setActiveTab} />
      {settings.error && (
        <Typography variant="body" color="distracting" className="text-xs">
          {settings.error}
        </Typography>
      )}
      <TabContent activeTab={activeTab} settings={settings} />
    </div>
  );
};

export { SettingsView };
