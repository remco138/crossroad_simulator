(ns ui.core
  (:require [figwheel.client :as fw :include-macros true]
            [reagent.core :as reagent :refer [atom]]
            [clojure.string :as string :refer [split-lines]]
            [cljsjs.paperjs]

            [ui.drawing :as drawing]
            ;[ui.cars]
            ))

(enable-console-print!)



(defn parse-svg [url]
  (let [item js/paper.project]
    (.importSVG item url)
    item))


(defn spawn-car [road-name]
  ())

(defn root-component []
  [:div
   [:canvas {:id "mycanvas" :width "967" :height "459"}]
   ;[:object {:id "svghere" :data "map3.svg" :type "image/svg+xml"}]
   [:button  {:on-click #(println "works")} "yeee boiii. add 5 cars"]
  ])

(reagent/render
  [root-component]
  (.-body js/document))


(defn init! []
  (js/paper.setup(js/document.getElementById "mycanvas"))
  (set! js/paper.view.onFrame (var drawing/on-frame))
  ;(parse-svg "map3.svg")
  (drawing/init!)
  )

(set! (.-onload js/window) (fn [] (init!)))


(fw/watch-and-reload
  :websocket-url   "ws://localhost:3449/figwheel-ws"
  :jsload-callback (fn []
                     (init!)
                     (print "reloaded")))

