(ns ui.core
  (:require

   [figwheel.client :as fw :include-macros true]
            [reagent.core :as reagent :refer [atom]]
            [clojure.string :as string :refer [split-lines]]
            [cljsjs.paperjs]

            [ui.state :as state]
            [ui.drawing :as drawing]
            [ui.cars :as cars]
            ))

(enable-console-print!)



(defn parse-svg [url]
  (let [item js/paper.project]
    (print (.importSVG item url))))


(defn spawn-car [road-name]
  ())

(defn root-component []
  [:div
   [:canvas {:id "mycanvas" :width "967" :height "459"}]
   [:button  {:on-click #(println (cars/random-car　(:roads @state/state)))} "yeee boiii. add a car"]
  ])

(reagent/render
  [root-component]
  (.-body js/document))


(defn init! []
  (js/paper.setup(js/document.getElementById "mycanvas"))
  (set! js/paper.view.onFrame (var drawing/on-frame))
  (drawing/init!)
  )

(set! (.-onload js/window) (fn [] (init!)))


(fw/watch-and-reload
  :websocket-url   "ws://localhost:3449/figwheel-ws"
  :jsload-callback (fn []
                     (init!)
                     (print "reloaded")))

