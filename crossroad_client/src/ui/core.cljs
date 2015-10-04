(ns ui.core
  (:require
   [figwheel.client :as fw :include-macros true]
   [reagent.core :as reagent :refer [atom]]
   [clojure.string :as string :refer [split-lines]]
   [cljsjs.paperjs]

   [ui.network :as network]
   [ui.state :as state]
   [ui.drawing :as drawing]
   [ui.cars :as cars]
   ))

(enable-console-print!)



(defn parse-svg [url]
  (let [item js/paper.project]
    (print (.importSVG item url))))

(defn spawn-car! []
  (let [car (state/add-car-channel! (cars/random-car　(:roads @state/state)))]
    (cars/car-ai (:chan car) car 0)))

(defn root-component []
  [:div
   [:canvas {:id "mycanvas" :width "967" :height "459"}]
   [:button  {:on-click spawn-car!} "yeee boiii. add a car"]
   [:button  {:on-click #(network/connect! 9990)} "(re)connect"]
   [:button  {:on-click #(network/send! "ayyyyy")} "send data"]

   [:input {:type "range"
            :value (:speed @state/ui-state)
            :min 0.1
            :max 10
            :step 0.1
            :name "speed-slider"
            :on-change (fn [x]
                         (swap! state/ui-state assoc :speed (-> x .-target .-value js/parseFloat)))}]
   [:span (:speed @state/ui-state)]

  ])

(reagent/render
  [root-component]
  (.-body js/document))


(defn init! []
  (js/paper.setup(js/document.getElementById "mycanvas"))
  (set! js/paper.view.onFrame (var drawing/on-frame))
  (network/connect! 9990)
  (state/init!)
  (drawing/init!))

(set! (.-onload js/window) (fn [] (init!)))


(fw/watch-and-reload
  :websocket-url   "ws://localhost:3449/figwheel-ws"
  :jsload-callback (fn []
                     (init!)
                     (print "reloaded")))

