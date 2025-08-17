use crate::router::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AdminCaptionsPageProps {}

#[function_component(AdminCaptionsPage)]
pub fn admin_captions_page(_props: &AdminCaptionsPageProps) -> Html {
    html! {
        <div class="min-h-screen bg-gray-700 p-4">
            <div class="max-w-6xl mx-auto">
                <div class="bg-white rounded-lg shadow-lg p-8">
                    <div class="flex justify-between items-center mb-6">
                        <h1 class="text-3xl font-bold text-gray-800">
                            {"Caption Management"}
                        </h1>
                        <div class="flex gap-4">
                            <Link<Route> to={Route::Admin} classes="text-blue-600 hover:underline">
                                {"← Back to Admin"}
                            </Link<Route>>
                            <Link<Route> to={Route::Home} classes="text-blue-600 hover:underline">
                                {"← Back to Search"}
                            </Link<Route>>
                        </div>
                    </div>

                    <div class="bg-gray-50 p-6 rounded-lg">
                        <h2 class="text-xl font-semibold text-gray-700 mb-4">
                            {"Caption Management System"}
                        </h2>
                        <p class="text-gray-600 mb-4">
                            {"This page will allow you to manage video captions, including:"}
                        </p>
                        <ul class="list-disc list-inside space-y-2 text-gray-600 mb-6">
                            <li>{"View and search through all stored captions"}</li>
                            <li>{"Edit caption text and timestamps"}</li>
                            <li>{"Delete individual caption segments"}</li>
                            <li>{"Re-process captions for specific videos"}</li>
                            <li>{"Import/export caption data"}</li>
                            <li>{"Caption quality analytics and statistics"}</li>
                        </ul>
                        <div class="bg-blue-50 border border-blue-200 p-4 rounded">
                            <p class="text-blue-800 font-medium">
                                {"🚧 Coming Soon"}
                            </p>
                            <p class="text-blue-700 text-sm mt-1">
                                {"Caption management functionality is currently under development. Check back soon for full caption editing capabilities."}
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
